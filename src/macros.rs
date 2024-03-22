#[macro_export]
macro_rules! MK_BOOL {
    ($val:expr) => {
        BooleanValue {
            value: $val
        }
    }
}

#[macro_export]
macro_rules! MK_NULL {
    () => {
        NullValue {}
    };
}

#[macro_export]
macro_rules! MK_NUMBER {
    ($val:expr) => {
        NumberValue {
            value: $val
        }
    };
}

#[macro_export]
macro_rules! MK_NATIVE_FN {
    ($function:expr) => {
        NativeFnValue {
            call: FunctionCall {
                func: Rc::new($function)
            }
        }
    };
}

#[macro_export]
macro_rules! MK_STRING {
    ($val:expr) => {
        StringValue {
            value: $val
        }
    };
}