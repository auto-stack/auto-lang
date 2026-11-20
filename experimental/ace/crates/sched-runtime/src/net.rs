//! WinSock transport for the scheduler's checked platform.net/1 calls.
//! The emitted caller holds the platform lock throughout each operation.
//! The native platform ledger owns handles, bounds and last-error state.

use core::mem::{offset_of, size_of};
use core::ptr::{addr_of, addr_of_mut, null_mut};
use windows_sys::Win32::Networking::WinSock::*;

const INVALID_BOUNDS: i32 = -4;
const INVALID_VALUE: i32 = -7;
const OS_FAILED: i32 = -8;
const BUDGET: i32 = -9;
const RETRY: i32 = -10;

const MAX_SOCKETS: u32 = 16;
const MAX_CALL_BYTES: i32 = 65_536;
const MAX_RUN_BYTES: i32 = 1_048_576;
const CONNECT_MS: i32 = 5000;
const WRITE_MS: i32 = 5000;
const ACCEPT_MS: i32 = 50;
const READ_MS: i32 = 500;

const KIND_MEM: i32 = 2;
const KIND_SOCKET: i32 = 6;
const KIND_LISTENER: i32 = 7;

// Private mn-platform-shim ABI. These offsets describe the same ledger
// traversed by mn_pio_local_find_slot; the other kinds stay opaque here.
#[repr(C)]
struct Record {
    kind: i32,
    released: i32,
    generation: i32,
    borrow_count: i32,
    address: usize,
    logical_len: u64,
    owner_fields: [u8; 64],
}

#[repr(C)]
struct Header {
    next_gen: i32,
    last_os_error: i32,
    accounting: [u8; 24],
    initialized_slots: i32,
    padding: [u8; 28],
}

const _: () = {
    assert!(size_of::<Header>() == 64);
    assert!(size_of::<Record>() == 96);
    assert!(offset_of!(Record, address) == 16);
    assert!(offset_of!(Record, logical_len) == 24);
};

extern "system" {
    fn mn_pio_state_address() -> *mut Header;
    fn mn_pio_local_find_slot(handle: i32, kind: i32) -> isize;
    fn mn_pio_local_alloc_slot(
        kind: i32,
        address: usize,
        logical: u64,
        mapped: u64,
        extra: i32,
    ) -> i32;
    fn GetTickCount64() -> u64;
}

static mut WINSOCK_STARTED: bool = false;
static mut STREAMS: u32 = 0;
static mut BYTES: i32 = 0;

unsafe fn os_error(error: i32) -> i32 {
    let header = mn_pio_state_address();
    addr_of_mut!((*header).last_os_error).write(error);
    OS_FAILED
}

unsafe fn last_error() -> i32 {
    os_error(WSAGetLastError())
}

unsafe fn start_winsock() -> Result<(), i32> {
    if !WINSOCK_STARTED {
        let mut data = WSADATA::default();
        let code = WSAStartup(0x0202, &mut data);
        if code != 0 {
            return Err(os_error(code));
        }
        WINSOCK_STARTED = true;
    }
    Ok(())
}

unsafe fn record(handle: i32, kind: i32) -> Result<*mut Record, i32> {
    let found = mn_pio_local_find_slot(handle, kind);
    if found <= 0 {
        Err(found as i32)
    } else {
        Ok(found as *mut Record)
    }
}

unsafe fn own(socket: SOCKET, kind: i32) -> i32 {
    let result = mn_pio_local_alloc_slot(kind, socket, 0, 0, 0);
    if result < 0 {
        closesocket(socket);
    } else if kind == KIND_SOCKET {
        STREAMS += 1;
    }
    result
}

unsafe fn make_nonblocking(socket: SOCKET) -> Result<(), i32> {
    let mut enable = 1;
    if ioctlsocket(socket, FIONBIO, &mut enable) == 0 {
        Ok(())
    } else {
        Err(last_error())
    }
}

unsafe fn ready(socket: SOCKET, events: i16, ms: i32) -> Result<(), i32> {
    let mut fd = WSAPOLLFD {
        fd: socket,
        events,
        revents: 0,
    };
    match WSAPoll(&mut fd, 1, ms) {
        n if n > 0 => Ok(()),
        0 => Err(RETRY),
        _ => Err(last_error()),
    }
}

unsafe fn resolve_address(host: *const u8, len: i32, port: i32) -> Result<*mut ADDRINFOW, i32> {
    if len <= 0 || len > 256 || !(0..=65535).contains(&port) {
        return Err(INVALID_VALUE);
    }
    let host = core::slice::from_raw_parts(host, len as usize);
    if host.contains(&0) {
        return Err(INVALID_VALUE);
    }
    let text = core::str::from_utf8(host).map_err(|_| INVALID_VALUE)?;
    // An UTF-8 host of up to 256 bytes always fits in 256 UTF-16 units.
    // Surrounding brackets are the IPv6 form accepted by the source API.
    let text = text
        .strip_prefix('[')
        .and_then(|text| text.strip_suffix(']'))
        .unwrap_or(text);
    let mut wide = [0u16; 257];
    for (index, unit) in text.encode_utf16().enumerate() {
        wide.as_mut_ptr().add(index).write(unit);
    }
    let mut service = [0u16; 6];
    let mut port = port as u32;
    let mut index = 5;
    loop {
        index -= 1;
        service
            .as_mut_ptr()
            .add(index)
            .write(b'0' as u16 + (port % 10) as u16);
        port /= 10;
        if port == 0 {
            break;
        }
    }
    let hints = ADDRINFOW {
        ai_socktype: SOCK_STREAM,
        ai_protocol: IPPROTO_TCP,
        ai_flags: AI_NUMERICSERV as i32,
        ..ADDRINFOW::default()
    };
    let mut resolved = null_mut();
    let error = GetAddrInfoW(
        wide.as_ptr(),
        service.as_ptr().add(index),
        &hints,
        &mut resolved,
    );
    if error != 0 || resolved.is_null() {
        return Err(os_error(if error == 0 { WSAHOST_NOT_FOUND } else { error }));
    }
    Ok(resolved)
}

unsafe fn establish(socket: SOCKET, address: &ADDRINFOW) -> Result<(), i32> {
    make_nonblocking(socket)?;
    if connect(socket, address.ai_addr, address.ai_addrlen as i32) == 0 {
        return Ok(());
    }
    let error = WSAGetLastError();
    if error != WSAEWOULDBLOCK {
        return Err(os_error(error));
    }
    ready(socket, POLLWRNORM, CONNECT_MS).map_err(|code| {
        if code == RETRY {
            os_error(WSAETIMEDOUT)
        } else {
            code
        }
    })?;
    let mut pending = 0i32;
    let mut bytes = size_of::<i32>() as i32;
    if getsockopt(
        socket,
        SOL_SOCKET,
        SO_ERROR,
        addr_of_mut!(pending).cast(),
        &mut bytes,
    ) != 0
    {
        Err(last_error())
    } else if pending != 0 {
        Err(os_error(pending))
    } else {
        Ok(())
    }
}

#[no_mangle]
pub unsafe extern "system" fn mn_pio_net_connect(host: *const u8, len: i32, port: i32) -> i32 {
    if STREAMS >= MAX_SOCKETS {
        return BUDGET;
    }
    if let Err(code) = start_winsock() {
        return code;
    }
    let address = match resolve_address(host, len, port) {
        Ok(address) => address,
        Err(code) => return code,
    };
    let socket = socket((*address).ai_family, SOCK_STREAM, IPPROTO_TCP);
    let result = if socket == INVALID_SOCKET {
        last_error()
    } else {
        match establish(socket, &*address) {
            Ok(()) => own(socket, KIND_SOCKET),
            Err(code) => {
                closesocket(socket);
                code
            }
        }
    };
    FreeAddrInfoW(address);
    result
}

#[no_mangle]
pub unsafe extern "system" fn mn_pio_net_listen(port: i32) -> i32 {
    if !(0..=65535).contains(&port) {
        return INVALID_VALUE;
    }
    if let Err(code) = start_winsock() {
        return code;
    }
    let socket = socket(i32::from(AF_INET), SOCK_STREAM, IPPROTO_TCP);
    if socket == INVALID_SOCKET {
        return last_error();
    }
    let address = SOCKADDR_IN {
        sin_family: AF_INET,
        sin_port: (port as u16).to_be(),
        sin_addr: IN_ADDR {
            S_un: IN_ADDR_0 {
                S_addr: u32::from_ne_bytes([127, 0, 0, 1]),
            },
        },
        sin_zero: [0; 8],
    };
    let bound = bind(
        socket,
        addr_of!(address).cast(),
        size_of::<SOCKADDR_IN>() as i32,
    );
    let result = if bound != 0 || listen(socket, SOMAXCONN as i32) != 0 {
        Err(last_error())
    } else {
        make_nonblocking(socket)
    };
    match result {
        Ok(()) => own(socket, KIND_LISTENER),
        Err(code) => {
            closesocket(socket);
            code
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn mn_pio_net_local_port(handle: i32) -> i32 {
    let listener = match record(handle, KIND_LISTENER) {
        Ok(slot) => (*slot).address,
        Err(code) => return code,
    };
    let mut address = SOCKADDR_IN::default();
    let mut size = size_of::<SOCKADDR_IN>() as i32;
    if getsockname(listener, addr_of_mut!(address).cast(), &mut size) != 0 {
        return last_error();
    }
    i32::from(u16::from_be(address.sin_port))
}

#[no_mangle]
pub unsafe extern "system" fn mn_pio_net_accept(handle: i32) -> i32 {
    if STREAMS >= MAX_SOCKETS {
        return BUDGET;
    }
    let listener = match record(handle, KIND_LISTENER) {
        Ok(slot) => (*slot).address,
        Err(code) => return code,
    };
    if let Err(code) = ready(listener, POLLRDNORM, ACCEPT_MS) {
        return code;
    }
    let accepted = accept(listener, null_mut(), null_mut());
    if accepted == INVALID_SOCKET {
        let code = WSAGetLastError();
        return if code == WSAEWOULDBLOCK {
            RETRY
        } else {
            os_error(code)
        };
    }
    match make_nonblocking(accepted) {
        Ok(()) => own(accepted, KIND_SOCKET),
        Err(code) => {
            closesocket(accepted);
            code
        }
    }
}

unsafe fn check_budget(len: i32) -> Result<(), i32> {
    if !(0..=MAX_CALL_BYTES).contains(&len) || BYTES > MAX_RUN_BYTES - len {
        Err(BUDGET)
    } else {
        Ok(())
    }
}

unsafe fn region_buffer(handle: i32, off: i32, len: i32) -> Result<*mut u8, i32> {
    let slot = record(handle, KIND_MEM)?;
    if off < 0 || len < 0 || off as u64 + len as u64 > (*slot).logical_len {
        return Err(INVALID_BOUNDS);
    }
    Ok(((*slot).address as *mut u8).add(off as usize))
}

unsafe fn write_bytes(socket: SOCKET, buf: *const u8, len: i32) -> i32 {
    let started = GetTickCount64();
    let mut total = 0;
    while total < len {
        let sent = send(socket, buf.add(total as usize), len - total, 0);
        if sent > 0 {
            total += sent;
            continue;
        }
        let code = if sent == 0 {
            WSAECONNRESET
        } else {
            WSAGetLastError()
        };
        if code != WSAEWOULDBLOCK {
            os_error(code);
            break;
        }
        let left = u64::from(WRITE_MS as u32).saturating_sub(GetTickCount64() - started);
        if left == 0 || ready(socket, POLLWRNORM, left as i32).is_err() {
            os_error(WSAETIMEDOUT);
            break;
        }
    }
    BYTES += total;
    if total == 0 && len != 0 {
        OS_FAILED
    } else {
        total
    }
}

#[no_mangle]
pub unsafe extern "system" fn mn_pio_net_write_some(handle: i32, buf: *const u8, len: i32) -> i32 {
    let socket = match record(handle, KIND_SOCKET) {
        Ok(slot) => (*slot).address,
        Err(code) => return code,
    };
    if let Err(code) = check_budget(len) {
        return code;
    }
    write_bytes(socket, buf, len)
}

unsafe fn with_region(
    handle: i32,
    region: i32,
    off: i32,
    len: i32,
    transfer: unsafe fn(SOCKET, *mut u8, i32) -> i32,
) -> i32 {
    if off < 0 || len < 0 {
        return INVALID_BOUNDS;
    }
    let result = (|| {
        let socket = (*record(handle, KIND_SOCKET)?).address;
        let buf = region_buffer(region, off, len)?;
        check_budget(len)?;
        Ok(transfer(socket, buf, len))
    })();
    result.unwrap_or_else(|code| code)
}

unsafe fn read_bytes(socket: SOCKET, buf: *mut u8, len: i32) -> i32 {
    if len == 0 {
        return 0;
    }
    if let Err(code) = ready(socket, POLLRDNORM, READ_MS) {
        return code;
    }
    let received = recv(socket, buf, len, 0);
    if received < 0 {
        let code = WSAGetLastError();
        return if code == WSAEWOULDBLOCK {
            RETRY
        } else {
            os_error(code)
        };
    }
    BYTES += received;
    received
}

#[no_mangle]
pub unsafe extern "system" fn mn_pio_net_read_into(
    handle: i32,
    region: i32,
    off: i32,
    len: i32,
) -> i32 {
    with_region(handle, region, off, len, read_bytes)
}

#[no_mangle]
pub unsafe extern "system" fn mn_pio_net_write_region(
    handle: i32,
    region: i32,
    off: i32,
    len: i32,
) -> i32 {
    with_region(handle, region, off, len, |socket, buf, len| unsafe {
        write_bytes(socket, buf, len)
    })
}

#[no_mangle]
pub unsafe extern "system" fn mn_pio_net_close(handle: i32) -> i32 {
    let slot = match record(handle, KIND_SOCKET) {
        Ok(slot) => slot,
        Err(code) => return code,
    };
    if closesocket((*slot).address) != 0 {
        return last_error();
    }
    STREAMS -= 1;
    (*slot).released = 1;
    0
}

// The wrapper joins every task before calling this. Listeners have no
// explicit close operation in the frozen surface; the program owns them.
// Open streams are reclaimed too, but report a residue for a missed close.
#[no_mangle]
pub unsafe extern "system" fn mn_sched_net_drain() -> i32 {
    let header = mn_pio_state_address();
    let records = header.add(1).cast::<Record>();
    let mut residue = 0;
    for index in 0..(*header).initialized_slots as usize {
        let slot = &mut *records.add(index);
        if slot.released == 0 && matches!(slot.kind, KIND_SOCKET | KIND_LISTENER) {
            if closesocket(slot.address) != 0 || slot.kind == KIND_SOCKET {
                residue = 1;
            }
            slot.released = 1;
        }
    }
    if WINSOCK_STARTED {
        if WSACleanup() != 0 {
            residue = 1;
        }
        WINSOCK_STARTED = false;
    }
    residue
}
