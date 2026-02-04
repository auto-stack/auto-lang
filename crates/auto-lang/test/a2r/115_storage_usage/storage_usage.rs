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

fn main() {

    let heap: Heap = Heap.new();
    let heap_data = heap.data();
    let heap_cap = heap.capacity();
    let can_grow_heap = heap.try_grow(100);


    let inline: InlineInt64 = InlineInt64.new();
    let inline_data = inline.data();
    let inline_cap = inline.capacity();
    let can_grow_inline = inline.try_grow(50);

    return 0;
}
