use enumify_struct::{enumify_struct, Applicable, ResolveToBase};

trait ResolveRef {
    type Base;
    fn resolve_ref(reference: &str) -> Self::Base;
}

#[derive(Debug, PartialEq, Clone)]
enum NotGeneric {
    A(i32),
    B(i32),
}

#[derive(Debug, PartialEq, Clone)]
enum ReferentialEnum<T> {
    Reference(String),
    Value(T),
}

#[derive(Debug, PartialEq, Clone)]
#[enumify_struct(ReferentialEnum)]
struct LowerStruct {
    a_prime: String,
    b_prime: NotGeneric,
    c_prime: String,
}

#[enumify_struct(ReferentialEnum)]
struct HigherStruct {
    a: i32,
    #[enumify_rename(EnumifiedLowerStruct)]
    #[enumify_wrap]
    b: LowerStruct,
    c: NotGeneric,
    d: i32,
    f: ReferentialEnum<String>,
}

impl<T: Clone + ResolveRef<Base = T>> ResolveToBase<T> for ReferentialEnum<T> {
    fn resolve_to_base(&self) -> T {
        match self {
            ReferentialEnum::Reference(ref_str) => T::resolve_ref(ref_str),
            ReferentialEnum::Value(v) => v.clone(),
        }
    }
}

// Each type used on a Struct that's enumified with ReferentialEnum must
// implement ResolveRef as required by the implementation of ResolveToBase for
// ReferentialEnum
impl ResolveRef for String {
    type Base = String;
    fn resolve_ref(reference: &str) -> Self::Base {
        reference.to_string()
    }
}

impl ResolveRef for i32 {
    type Base = i32;
    fn resolve_ref(reference: &str) -> Self::Base {
        reference.parse().unwrap_or(0)
    }
}

impl ResolveRef for NotGeneric {
    type Base = NotGeneric;
    fn resolve_ref(reference: &str) -> Self::Base {
        match reference {
            "A" => NotGeneric::A(1),
            "B" => NotGeneric::B(2),
            _ => panic!("Unknown reference"),
        }
    }
}

impl ResolveRef for EnumifiedLowerStruct {
    type Base = EnumifiedLowerStruct;
    fn resolve_ref(reference: &str) -> Self::Base {
        match reference {
            "A" => EnumifiedLowerStruct {
                a_prime: ReferentialEnum::Value("hello".into()),
                b_prime: ReferentialEnum::Value(NotGeneric::A(1)),
                c_prime: ReferentialEnum::Value("foo".into()),
            },
            "B" => EnumifiedLowerStruct {
                a_prime: ReferentialEnum::Value("world".into()),
                b_prime: ReferentialEnum::Value(NotGeneric::B(2)),
                c_prime: ReferentialEnum::Value("bar".into()),
            },
            _ => panic!("Unknown reference"),
        }
    }
}

#[test]
fn test_nested_enum() {
    let base_struct = HigherStruct {
        a: 1,
        b: LowerStruct {
            a_prime: "hello".into(),
            b_prime: NotGeneric::A(1),
            c_prime: "foo".into(),
        },
        c: NotGeneric::A(1),
        d: 4,
        f: ReferentialEnum::Value("world".into()),
    };

    let enumed_struct = EnumifiedHigherStruct {
        a: ReferentialEnum::Value(2),
        b: ReferentialEnum::Value(EnumifiedLowerStruct {
            a_prime: ReferentialEnum::Value("hello".into()),
            b_prime: ReferentialEnum::Value(NotGeneric::A(1)),
            c_prime: ReferentialEnum::Value("foo".into()),
        }),
        c: ReferentialEnum::Reference("B".into()),
        d: ReferentialEnum::Value(4),
        f: ReferentialEnum::Reference("world".into()),
    };

    let rebuilt_struct = enumed_struct.build(base_struct);
    assert_eq!(rebuilt_struct.a, 2);
    assert_eq!(rebuilt_struct.b.a_prime, "hello");
    assert_eq!(rebuilt_struct.b.b_prime, NotGeneric::A(1));
    assert_eq!(rebuilt_struct.b.c_prime, "foo");
    assert_eq!(rebuilt_struct.c, NotGeneric::B(2));
    assert_eq!(rebuilt_struct.d, 4);
    assert_eq!(rebuilt_struct.f, ReferentialEnum::Reference("world".into()));
}
