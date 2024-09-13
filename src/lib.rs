pub use enumify_macro::enumify_struct;

#[derive(Debug, PartialEq, Clone)]
enum NotGeneric {
    A(i32),
    B(i32),
}

#[derive(Debug, PartialEq, Clone)]
enum Generic<T> {
    Reference(String),
    Value(T),
}

#[derive(Debug, PartialEq, Clone)]
#[enumify_struct(Generic)]
struct LowerStruct {
    a_prime: String,
    b_prime: NotGeneric,
    c_prime: Option<NotGeneric>,
}

#[enumify_struct(Generic)]
struct ABCDE {
    a: i32,
    #[enumify_rename(EnumifiedLowerStruct)]
    #[enumify_wrap]
    b: LowerStruct,
    c: NotGeneric,
    d: Option<i32>,
    f: Generic<String>,
}

fn test() {
    let final_struct = EnumifiedABCDE {
        a: Generic::Value(1),
        b: Generic::Value(EnumifiedLowerStruct {
            a_prime: Generic::Value("hello".into()),
            b_prime: Generic::Value(NotGeneric::A(1)),
            c_prime: Generic::Value(Some(NotGeneric::B(2))),
        }),
        c: Generic::Reference("world".into()),
        d: Generic::Value(None),
        f: Generic::Value("world".into()),
    };
}
