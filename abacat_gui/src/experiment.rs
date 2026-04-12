use cxx::{ExternType, kind};

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("experiment.hpp");
        type BasicSuper = super::BasicSuper;

        include!("experiment.hpp");
        type BasicSub = super::BasicSub;

        include!("experiment.hpp");
        pub fn create_basic_sub(xyz: i32) -> BasicSub;

        include!("experiment.hpp");
        pub fn print_basic_sub(sub: &BasicSub);

        include!("experiment.hpp");
        pub fn test_func();
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BasicSuper {
    pub smelly: i32,
    pub sub: i32,
}

#[derive(Debug)]
#[repr(C)]
pub struct BasicSub {
    pub smelly: i32,
    pub sub: i32,
}

unsafe impl ExternType for BasicSub {
    type Id = cxx::type_id!("BasicSub");
    type Kind = kind::Trivial;
}

unsafe impl ExternType for BasicSuper {
    type Id = cxx::type_id!("BasicSuper");
    type Kind = kind::Trivial;
}
