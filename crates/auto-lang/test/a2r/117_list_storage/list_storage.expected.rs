trait Storage<T> {
    fn data(&self) -> *mut T;
    fn capacity(&self) -> /* unknown */;
    fn try_grow(&self, min_cap: /* unknown */) -> bool;
}


struct Heap<T> {
    ptr: *mut T,
    cap: /* unknown */,
}

impl Storage<T> for Heap<T> {
}

struct InlineInt64 {
    buffer: [i32; 64],
}

impl Storage<T> for InlineInt64 {
}

struct List<T, S> {
    len: /* unknown */,
    store: S,
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
