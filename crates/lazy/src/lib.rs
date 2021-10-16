pub trait Thunk<T, C>: Fn(C) -> T {}
impl<C, R, F: Fn(C) -> R> Thunk<R, C> for F {}

pub struct Lazy<T, C> {
    thunk: Box<dyn Thunk<T, C>>,
    val: Option<T>,
}

impl<T, C> Lazy<T, C> {
    pub fn new(thunk: Box<dyn Thunk<T, C>>) -> Self {
        Lazy { thunk, val: None }
    }

    pub fn try_init(&mut self, ctx: C) {
        if self.val.is_none() {
            self.val = Some((self.thunk)(ctx))
        }
    }

    pub fn get(&mut self, ctx: C) -> &T {
        self.try_init(ctx);
        self.val.as_ref().unwrap()
    }

    pub fn get_mut(&mut self, ctx: C) -> &mut T {
        self.try_init(ctx);
        self.val.as_mut().unwrap()
    }
    
    pub fn set(&mut self, val: T) {
        self.val = Some(val)
    }

    pub fn take(&mut self) -> Option<T> {
        self.val.take()
    }
}
