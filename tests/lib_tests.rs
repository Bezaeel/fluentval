use fluentval::*;

// ── ValidationError ──────────────────────────────────────────────────────────

#[test]
fn test_validation_error_new() {
    let error = ValidationError::new("email", "must be a valid email");
    assert_eq!(error.property, "email");
    assert_eq!(error.message, "must be a valid email");
}

#[test]
fn test_validation_error_display() {
    let error = ValidationError::new("name", "must not be empty");
    assert_eq!(format!("{}", error), "name: must not be empty");
}

// ── ValidationResult ─────────────────────────────────────────────────────────

#[test]
fn test_validation_result_new() {
    let result = ValidationResult::new();
    assert!(result.is_valid());
    assert_eq!(result.errors().len(), 0);
}

#[test]
fn test_validation_result_default() {
    let result = ValidationResult::default();
    assert!(result.is_valid());
}

#[test]
fn test_validation_result_add_error() {
    let mut result = ValidationResult::new();
    result.add_error(ValidationError::new("email", "invalid email"));
    assert!(!result.is_valid());
    assert_eq!(result.errors().len(), 1);
}

#[test]
fn test_validation_result_add_errors() {
    let mut result = ValidationResult::new();
    result.add_errors(vec![
        ValidationError::new("email", "invalid email"),
        ValidationError::new("name", "must not be empty"),
    ]);
    assert!(!result.is_valid());
    assert_eq!(result.errors().len(), 2);
}

#[test]
fn test_validation_result_errors_by_property() {
    let mut result = ValidationResult::new();
    result.add_error(ValidationError::new("email", "invalid email"));
    result.add_error(ValidationError::new("email", "must not be empty"));
    result.add_error(ValidationError::new("name", "too short"));

    let grouped = result.errors_by_property();
    assert_eq!(grouped.len(), 2);
    assert_eq!(grouped.get("email").unwrap().len(), 2);
    assert_eq!(grouped.get("name").unwrap().len(), 1);
}

#[test]
fn test_validation_result_first_error_for() {
    let mut result = ValidationResult::new();
    result.add_error(ValidationError::new("email", "first error"));
    result.add_error(ValidationError::new("email", "second error"));
    result.add_error(ValidationError::new("name", "name error"));

    assert_eq!(result.first_error_for("email"), Some("first error"));
    assert_eq!(result.first_error_for("name"), Some("name error"));
    assert_eq!(result.first_error_for("nonexistent"), None);
}

// ── RuleBuilder: value-only via identity accessor ────────────────────────────

#[test]
fn test_rule_builder_not_empty() {
    let rule_fn = RuleBuilder::for_property("name", |s: &String| s)
        .not_empty(None::<String>)
        .build();

    assert!(!rule_fn(&"".to_string()).is_empty());
    assert!(!rule_fn(&"   ".to_string()).is_empty());
    assert!(rule_fn(&"valid".to_string()).is_empty());
}

#[test]
fn test_rule_builder_min_length() {
    let rule_fn = RuleBuilder::for_property("name", |s: &String| s)
        .min_length(5, None::<String>)
        .build();

    assert!(!rule_fn(&"abc".to_string()).is_empty());
    assert!(rule_fn(&"abcde".to_string()).is_empty());
    assert!(rule_fn(&"abcdef".to_string()).is_empty());
}

#[test]
fn test_rule_builder_max_length() {
    let rule_fn = RuleBuilder::for_property("name", |s: &String| s)
        .max_length(5, None::<String>)
        .build();

    assert!(rule_fn(&"abc".to_string()).is_empty());
    assert!(rule_fn(&"abcde".to_string()).is_empty());
    assert!(!rule_fn(&"abcdef".to_string()).is_empty());
}

#[test]
fn test_rule_builder_length() {
    let rule_fn = RuleBuilder::for_property("name", |s: &String| s)
        .length(3, 5, None::<String>, None::<String>)
        .build();

    assert!(!rule_fn(&"ab".to_string()).is_empty());
    assert!(rule_fn(&"abc".to_string()).is_empty());
    assert!(rule_fn(&"abcde".to_string()).is_empty());
    assert!(!rule_fn(&"abcdef".to_string()).is_empty());
}

#[test]
fn test_rule_builder_email() {
    let rule_fn = RuleBuilder::for_property("email", |s: &String| s)
        .email(None::<String>)
        .build();

    assert!(!rule_fn(&"invalid".to_string()).is_empty());
    assert!(rule_fn(&"test@example.com".to_string()).is_empty());
    assert!(rule_fn(&"user.name@domain.co.uk".to_string()).is_empty());
    assert!(!rule_fn(&"@example.com".to_string()).is_empty());
}

#[test]
fn test_rule_builder_greater_than() {
    let rule_fn = RuleBuilder::for_property("age", |n: &i32| n)
        .greater_than(18, None::<String>)
        .build();

    assert!(!rule_fn(&17).is_empty());
    assert!(!rule_fn(&18).is_empty());
    assert!(rule_fn(&19).is_empty());
}

#[test]
fn test_rule_builder_greater_than_or_equal() {
    let rule_fn = RuleBuilder::for_property("age", |n: &i32| n)
        .greater_than_or_equal(18, None::<String>)
        .build();

    assert!(!rule_fn(&17).is_empty());
    assert!(rule_fn(&18).is_empty());
    assert!(rule_fn(&19).is_empty());
}

#[test]
fn test_rule_builder_less_than() {
    let rule_fn = RuleBuilder::for_property("age", |n: &i32| n)
        .less_than(65, None::<String>)
        .build();

    assert!(rule_fn(&64).is_empty());
    assert!(!rule_fn(&65).is_empty());
    assert!(!rule_fn(&66).is_empty());
}

#[test]
fn test_rule_builder_less_than_or_equal() {
    let rule_fn = RuleBuilder::for_property("age", |n: &i32| n)
        .less_than_or_equal(65, None::<String>)
        .build();

    assert!(rule_fn(&64).is_empty());
    assert!(rule_fn(&65).is_empty());
    assert!(!rule_fn(&66).is_empty());
}

#[test]
fn test_rule_builder_inclusive_between() {
    let rule_fn = RuleBuilder::for_property("age", |n: &i32| n)
        .inclusive_between(18, 65, None::<String>)
        .build();

    assert!(!rule_fn(&17).is_empty());
    assert!(rule_fn(&18).is_empty());
    assert!(rule_fn(&50).is_empty());
    assert!(rule_fn(&65).is_empty());
    assert!(!rule_fn(&66).is_empty());
}

#[test]
fn test_rule_builder_must_value_only() {
    let rule_fn = RuleBuilder::for_property("password", |s: &String| s)
        .must(|_, s| s.len() >= 8, "must be at least 8 characters")
        .build();

    assert!(!rule_fn(&"short".to_string()).is_empty());
    assert!(rule_fn(&"longenough".to_string()).is_empty());
}

#[test]
fn test_rule_builder_not_null() {
    let rule_fn = RuleBuilder::for_property("value", |o: &Option<String>| o)
        .not_null(None::<String>)
        .build();

    assert!(!rule_fn(&None::<String>).is_empty());
    assert!(rule_fn(&Some("value".to_string())).is_empty());
}

#[test]
fn test_rule_builder_chaining() {
    let rule_fn = RuleBuilder::for_property("name", |s: &String| s)
        .not_empty(None::<String>)
        .min_length(3, None::<String>)
        .max_length(10, None::<String>)
        .build();

    assert!(!rule_fn(&"".to_string()).is_empty());
    assert!(!rule_fn(&"ab".to_string()).is_empty());
    assert!(rule_fn(&"abc".to_string()).is_empty());
    assert!(rule_fn(&"abcdefghij".to_string()).is_empty());
    assert!(!rule_fn(&"abcdefghijk".to_string()).is_empty());
}

#[test]
fn test_rule_builder_custom_rule() {
    let rule_fn = RuleBuilder::for_property("value", |s: &String| s)
        .rule(|v| {
            if v.contains("forbidden") {
                Some("contains forbidden word".to_string())
            } else {
                None
            }
        })
        .build();

    assert!(!rule_fn(&"forbidden word".to_string()).is_empty());
    assert!(rule_fn(&"allowed word".to_string()).is_empty());
}

#[test]
fn test_rule_builder_custom_messages() {
    let rule_fn = RuleBuilder::for_property("name", |s: &String| s)
        .not_empty(Some("custom not empty"))
        .build();
    assert_eq!(rule_fn(&"".to_string())[0].message, "custom not empty");

    let rule_fn = RuleBuilder::for_property("val", |o: &Option<String>| o)
        .not_null(Some("custom not null"))
        .build();
    assert_eq!(rule_fn(&None::<String>)[0].message, "custom not null");

    let rule_fn = RuleBuilder::for_property("name", |s: &String| s)
        .min_length(5, Some("custom min length"))
        .build();
    assert_eq!(rule_fn(&"abc".to_string())[0].message, "custom min length");

    let rule_fn = RuleBuilder::for_property("name", |s: &String| s)
        .max_length(3, Some("custom max length"))
        .build();
    assert_eq!(rule_fn(&"abcdef".to_string())[0].message, "custom max length");

    let rule_fn = RuleBuilder::for_property("email", |s: &String| s)
        .email(Some("custom email error"))
        .build();
    assert_eq!(rule_fn(&"invalid".to_string())[0].message, "custom email error");

    let rule_fn = RuleBuilder::for_property("age", |n: &i32| n)
        .greater_than(18, Some("custom greater than"))
        .build();
    assert_eq!(rule_fn(&10)[0].message, "custom greater than");

    let rule_fn = RuleBuilder::for_property("age", |n: &i32| n)
        .greater_than_or_equal(18, Some("custom gte"))
        .build();
    assert_eq!(rule_fn(&10)[0].message, "custom gte");

    let rule_fn = RuleBuilder::for_property("age", |n: &i32| n)
        .less_than(65, Some("custom less than"))
        .build();
    assert_eq!(rule_fn(&100)[0].message, "custom less than");

    let rule_fn = RuleBuilder::for_property("age", |n: &i32| n)
        .less_than_or_equal(65, Some("custom lte"))
        .build();
    assert_eq!(rule_fn(&100)[0].message, "custom lte");

    let rule_fn = RuleBuilder::for_property("score", |n: &i32| n)
        .inclusive_between(0, 100, Some("custom between"))
        .build();
    assert_eq!(rule_fn(&150)[0].message, "custom between");
}

// ── RuleBuilder: struct-aware via for_property ───────────────────────────────

#[test]
fn test_rule_builder_for_property_struct_aware() {
    #[derive(Debug)]
    struct Request {
        primary_phone: String,
        alt_phone: String,
    }

    let rule_fn = RuleBuilder::for_property("alt_phone", |r: &Request| &r.alt_phone)
        .not_empty(None::<String>)
        .must(
            |r, v| v != &r.primary_phone,
            "alt phone cannot match primary",
        )
        .build();

    // Same number → fails
    let errs = rule_fn(&Request {
        primary_phone: "111".into(),
        alt_phone: "111".into(),
    });
    assert!(errs.iter().any(|e| e.property == "alt_phone"));

    // Different number → passes
    let errs = rule_fn(&Request {
        primary_phone: "111".into(),
        alt_phone: "222".into(),
    });
    assert!(errs.is_empty());
}

// ── Option-transparent rules ─────────────────────────────────────────────────

#[test]
fn test_option_string_not_empty() {
    let rule_fn = RuleBuilder::for_property("name", |s: &Option<String>| s)
        .not_empty(None::<String>)
        .build();

    assert!(rule_fn(&None::<String>).is_empty());                  // None passes
    assert!(!rule_fn(&Some("".to_string())).is_empty());            // Some("") fails
    assert!(!rule_fn(&Some("   ".to_string())).is_empty());         // whitespace fails
    assert!(rule_fn(&Some("hello".to_string())).is_empty());        // Some(non-empty) passes
}

#[test]
fn test_option_string_min_length_max_length_email() {
    let rule_fn = RuleBuilder::for_property("email", |s: &Option<String>| s)
        .min_length(3, None::<String>)
        .max_length(50, None::<String>)
        .email(None::<String>)
        .build();

    assert!(rule_fn(&None::<String>).is_empty());                       // None passes everything
    assert!(!rule_fn(&Some("a".into())).is_empty());                    // too short
    assert!(!rule_fn(&Some("nope".into())).is_empty());                 // not an email
    assert!(rule_fn(&Some("ok@example.com".into())).is_empty());        // valid
}

#[test]
fn test_option_numeric_greater_than() {
    let rule_fn = RuleBuilder::for_property("age", |a: &Option<usize>| a)
        .greater_than(18, None::<String>)
        .build();

    assert!(rule_fn(&None::<usize>).is_empty());     // None passes
    assert!(!rule_fn(&Some(10)).is_empty());          // 10 <= 18 fails
    assert!(!rule_fn(&Some(18)).is_empty());          // 18 <= 18 fails (exclusive)
    assert!(rule_fn(&Some(19)).is_empty());           // 19 > 18 passes
}

#[test]
fn test_option_numeric_full_range() {
    let rule_fn = RuleBuilder::for_property("score", |s: &Option<i32>| s)
        .greater_than_or_equal(0, None::<String>)
        .less_than_or_equal(100, None::<String>)
        .inclusive_between(0, 100, None::<String>)
        .build();

    assert!(rule_fn(&None::<i32>).is_empty());
    assert!(!rule_fn(&Some(-1)).is_empty());
    assert!(rule_fn(&Some(50)).is_empty());
    assert!(rule_fn(&Some(100)).is_empty());
    assert!(!rule_fn(&Some(101)).is_empty());
}

#[test]
fn test_option_compose_not_null_and_value_rules() {
    // Mirrors the canonical "if Married, age must be present AND >= 18" use case.
    #[derive(Debug)]
    struct Person {
        marital_status: String,
        age: Option<usize>,
    }

    let rule_fn = RuleBuilder::for_property("age", |p: &Person| &p.age)
        .not_null(Some("age is required for married person"))
        .greater_than_or_equal(18, Some("must be at least 18 years old"))
        .when(|p| p.marital_status == "Married")
        .build();

    // Married + None → not_null fires
    let errs = rule_fn(&Person {
        marital_status: "Married".into(),
        age: None,
    });
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].message, "age is required for married person");

    // Married + Some(15) → greater_than_or_equal fires
    let errs = rule_fn(&Person {
        marital_status: "Married".into(),
        age: Some(15),
    });
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].message, "must be at least 18 years old");

    // Married + Some(25) → passes
    let errs = rule_fn(&Person {
        marital_status: "Married".into(),
        age: Some(25),
    });
    assert!(errs.is_empty());

    // Single + None → condition false, all rules skipped
    let errs = rule_fn(&Person {
        marital_status: "Single".into(),
        age: None,
    });
    assert!(errs.is_empty());
}

// ── .when() conditional rule ─────────────────────────────────────────────────

#[test]
fn test_rule_builder_when_gates_chain() {
    #[derive(Debug)]
    struct Request {
        payment_method: String,
        billing_address: String,
    }

    let rule_fn = RuleBuilder::for_property("billing_address", |r: &Request| &r.billing_address)
        .not_empty(Some("billing address is required for card payments"))
        .when(|r| r.payment_method == "CARD")
        .build();

    // CARD + empty billing → fails
    let errs = rule_fn(&Request {
        payment_method: "CARD".into(),
        billing_address: "".into(),
    });
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].message, "billing address is required for card payments");

    // CASH + empty billing → condition false, no errors
    let errs = rule_fn(&Request {
        payment_method: "CASH".into(),
        billing_address: "".into(),
    });
    assert!(errs.is_empty());

    // CARD + non-empty billing → passes
    let errs = rule_fn(&Request {
        payment_method: "CARD".into(),
        billing_address: "123 Main St".into(),
    });
    assert!(errs.is_empty());
}

#[test]
fn test_rule_builder_when_multiple_calls_are_anded() {
    #[derive(Debug)]
    struct Order {
        country: String,
        priority: String,
        tracking_code: String,
    }

    let rule_fn = RuleBuilder::for_property("tracking_code", |o: &Order| &o.tracking_code)
        .not_empty(None::<String>)
        .when(|o| o.country == "US")
        .when(|o| o.priority == "EXPRESS")
        .build();

    // US + EXPRESS + empty → fails (both conditions true)
    let errs = rule_fn(&Order {
        country: "US".into(),
        priority: "EXPRESS".into(),
        tracking_code: "".into(),
    });
    assert!(!errs.is_empty());

    // US + STANDARD + empty → passes (priority condition false)
    let errs = rule_fn(&Order {
        country: "US".into(),
        priority: "STANDARD".into(),
        tracking_code: "".into(),
    });
    assert!(errs.is_empty());

    // UK + EXPRESS + empty → passes (country condition false)
    let errs = rule_fn(&Order {
        country: "UK".into(),
        priority: "EXPRESS".into(),
        tracking_code: "".into(),
    });
    assert!(errs.is_empty());
}

#[test]
fn test_rule_builder_when_position_in_chain_does_not_matter() {
    #[derive(Debug)]
    struct Form {
        kind: String,
        value: String,
    }

    // .when() before any rule
    let rule_before = RuleBuilder::for_property("value", |f: &Form| &f.value)
        .when(|f| f.kind == "required")
        .not_empty(None::<String>)
        .build();

    // .when() after all rules
    let rule_after = RuleBuilder::for_property("value", |f: &Form| &f.value)
        .not_empty(None::<String>)
        .when(|f| f.kind == "required")
        .build();

    let optional = Form { kind: "optional".into(), value: "".into() };
    let required = Form { kind: "required".into(), value: "".into() };

    assert!(rule_before(&optional).is_empty());
    assert!(rule_after(&optional).is_empty());
    assert!(!rule_before(&required).is_empty());
    assert!(!rule_after(&required).is_empty());
}

#[test]
fn test_fluent_validator_when() {
    #[derive(Debug)]
    struct Checkout {
        payment_method: String,
        billing_address: String,
    }

    let v = FluentValidator::<Checkout>::new();
    rule_for!(v, |c| &c.billing_address)
        .not_empty(Some("required for card payments"))
        .when(|c| c.payment_method == "CARD");

    let card = Checkout { payment_method: "CARD".into(), billing_address: "".into() };
    let cash = Checkout { payment_method: "CASH".into(), billing_address: "".into() };

    assert!(!v.validate(&card).is_valid());
    assert!(v.validate(&cash).is_valid());
}

// ── Numeric / OptionLike trait coverage ──────────────────────────────────────

#[test]
fn test_numeric_trait_implementations() {
    assert_eq!(5i8.to_f64(), 5.0);
    assert_eq!(10i32.to_f64(), 10.0);
    assert_eq!(20u32.to_f64(), 20.0);
    assert!((3.14f32.to_f64() - 3.14).abs() < 0.0001);
    assert_eq!(2.71f64.to_f64(), 2.71);
}

#[test]
fn test_numeric_trait_remaining_implementations() {
    assert_eq!(5i16.to_f64(), 5.0);
    assert_eq!(100i64.to_f64(), 100.0);
    assert_eq!(200u8.to_f64(), 200.0);
    assert_eq!(1000u16.to_f64(), 1000.0);
    assert_eq!(50000u64.to_f64(), 50000.0);
}

#[test]
fn test_option_like_trait() {
    let some: Option<String> = Some("value".to_string());
    let none: Option<String> = None;

    assert!(some.is_some());
    assert!(none.is_none());
}

// ── FluentValidator ──────────────────────────────────────────────────────────

#[test]
fn test_fluent_validator_basic_string_rules() {
    #[derive(Debug)]
    struct User {
        name: String,
        email: String,
    }

    let v = FluentValidator::<User>::new();
    rule_for!(v, |u| &u.name)
        .not_empty(None::<String>)
        .min_length(2, None::<String>);
    rule_for!(v, |u| &u.email)
        .not_empty(None::<String>)
        .email(None::<String>);

    let result = v.validate(&User {
        name: "Jo".into(),
        email: "jo@example.com".into(),
    });
    assert!(result.is_valid());

    let result = v.validate(&User {
        name: "".into(),
        email: "not-an-email".into(),
    });
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| e.property == "name"));
    assert!(result.errors().iter().any(|e| e.property == "email"));
}

#[test]
fn test_fluent_validator_explicit_form_no_macro() {
    #[derive(Debug)]
    struct User {
        name: String,
        email: String,
    }

    let v = FluentValidator::<User>::new();
    v.rule_for("name", |u| &u.name)
        .not_empty(None::<String>)
        .min_length(2, None::<String>);
    v.rule_for("email", |u| &u.email)
        .not_empty(None::<String>)
        .email(None::<String>);

    let result = v.validate(&User {
        name: "".into(),
        email: "bad".into(),
    });
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| e.property == "name"));
    assert!(result.errors().iter().any(|e| e.property == "email"));
}

#[test]
fn test_fluent_validator_numeric_rules() {
    #[derive(Debug)]
    struct Product {
        price: f64,
        quantity: i32,
    }

    let v = FluentValidator::<Product>::new();
    rule_for!(v, |p| &p.price)
        .greater_than(0.0, None::<String>)
        .less_than_or_equal(1000.0, None::<String>);
    rule_for!(v, |p| &p.quantity)
        .greater_than_or_equal(1, None::<String>)
        .inclusive_between(1, 100, None::<String>);

    let result = v.validate(&Product {
        price: 50.0,
        quantity: 10,
    });
    assert!(result.is_valid());

    let result = v.validate(&Product {
        price: -1.0,
        quantity: 0,
    });
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| e.property == "price"));
    assert!(result.errors().iter().any(|e| e.property == "quantity"));
}

#[test]
fn test_fluent_validator_must_cross_property() {
    #[derive(Debug)]
    struct Request {
        phone: String,
        alt_phone: String,
    }

    let v = FluentValidator::<Request>::new();
    rule_for!(v, |r| &r.phone).not_empty(None::<String>);
    rule_for!(v, |r| &r.alt_phone).not_empty(None::<String>).must(
        |req, val| val != &req.phone,
        "alternative phone must differ from primary",
    );

    let result = v.validate(&Request {
        phone: "111".into(),
        alt_phone: "222".into(),
    });
    assert!(result.is_valid());

    let result = v.validate(&Request {
        phone: "111".into(),
        alt_phone: "111".into(),
    });
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| e.property == "alt_phone"));
}

#[test]
fn test_fluent_validator_must_value_only() {
    #[derive(Debug)]
    struct Form {
        password: String,
    }

    let v = FluentValidator::<Form>::new();
    rule_for!(v, |f| &f.password).min_length(8, None::<String>).must(
        |_, p| p.chars().any(|c| c.is_ascii_digit()),
        "must contain a digit",
    );

    let result = v.validate(&Form {
        password: "Secret1!".into(),
    });
    assert!(result.is_valid());

    let result = v.validate(&Form {
        password: "NoDigits!".into(),
    });
    assert!(!result.is_valid());
    assert_eq!(result.errors()[0].property, "password");
}

#[test]
fn test_fluent_validator_rules_accumulate_across_statements() {
    #[derive(Debug)]
    struct User {
        name: String,
    }

    let v = FluentValidator::<User>::new();
    rule_for!(v, |u| &u.name).not_empty(None::<String>);
    rule_for!(v, |u| &u.name).max_length(5, None::<String>);

    assert!(v.validate(&User { name: "Ada".into() }).is_valid());
    assert!(!v.validate(&User { name: "".into() }).is_valid());
    assert!(!v
        .validate(&User {
            name: "TooLongName".into()
        })
        .is_valid());
}

#[test]
fn test_fluent_validator_empty_is_valid() {
    #[derive(Debug)]
    struct Foo {
        #[allow(dead_code)]
        value: i32,
    }

    let v = FluentValidator::<Foo>::new();
    assert!(v.validate(&Foo { value: 42 }).is_valid());
}

#[test]
fn test_fluent_validator_default() {
    #[derive(Debug)]
    struct Foo {
        #[allow(dead_code)]
        value: i32,
    }

    let v = FluentValidator::<Foo>::default();
    assert!(v.validate(&Foo { value: 0 }).is_valid());
}

#[test]
fn test_fluent_validator_custom_property_label() {
    #[derive(Debug)]
    struct Cmd {
        tax_id: String,
    }

    let v = FluentValidator::<Cmd>::new();
    v.rule_for("taxId", |c| &c.tax_id).not_empty(None::<String>);

    let result = v.validate(&Cmd { tax_id: "".into() });
    assert!(!result.is_valid());
    assert_eq!(result.errors()[0].property, "taxId");
}

#[test]
fn test_fluent_validator_nested_field_macro() {
    #[derive(Debug)]
    struct Address {
        city: String,
    }
    #[derive(Debug)]
    struct Person {
        address: Address,
    }

    let v = FluentValidator::<Person>::new();
    rule_for!(v, |p| &p.address.city).not_empty(None::<String>);

    let result = v.validate(&Person {
        address: Address { city: "".into() },
    });
    assert!(!result.is_valid());
    assert_eq!(result.errors()[0].property, "address.city");

    let result = v.validate(&Person {
        address: Address {
            city: "London".into(),
        },
    });
    assert!(result.is_valid());
}

#[test]
fn test_fluent_validator_custom_messages() {
    #[derive(Debug)]
    struct Form {
        name: String,
        age: i32,
    }

    let v = FluentValidator::<Form>::new();
    rule_for!(v, |f| &f.name).not_empty(Some("name is required"));
    rule_for!(v, |f| &f.age).greater_than_or_equal(18, Some("must be an adult"));

    let result = v.validate(&Form {
        name: "".into(),
        age: 15,
    });
    assert_eq!(result.first_error_for("name"), Some("name is required"));
    assert_eq!(result.first_error_for("age"), Some("must be an adult"));
}
