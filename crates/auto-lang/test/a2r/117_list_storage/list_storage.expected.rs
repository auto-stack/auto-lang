trait Storage {
    fn data(&self) -> *mut T;
    fn capacity(&self) -> /* unknown */;
    fn try_grow(&self, min_cap: /* unknown */) -> bool;
}


struct Heap {
    ptr: *mut T,
    cap: /* unknown */,
}

impl Heap {





}

impl Storage for Heap {
    fn data(&self) -> *mut () {
{
        }    }
    fn capacity(&self) -> /* unknown */ {
{
        }    }
    fn try_grow(&self, min_cap: /* unknown */) -> bool {
{
        }    }
}

struct InlineInt64 {
    buffer: [i32; 64],
}

impl InlineInt64 {





}

impl Storage for InlineInt64 {
    fn data(&self) -> *mut i32 {
{
        }    }
    fn capacity(&self) -> /* unknown */ {
{
        }    }
    fn try_grow(&self, min_cap: /* unknown */) -> bool {
{
        }    }
}

struct List {
    len: /* unknown */,
    store: S,
}

impl List {



}


fn main() {

    let heap_list: List = List<int, Heap>.new();
    let heap_len = heap_list.len();
    let heap_cap = heap_list.capacity();


    let inline_list: List = List<int, InlineInt64>.new();
    let inline_len = inline_list.len();
    let inline_cap = inline_list.capacity();

    return 0;
}
