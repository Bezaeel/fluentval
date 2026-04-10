use fluentval::*;

#[derive(Debug)]
struct CreateCustomerRequest {
    name: String,
    email: String,
    phone: String,
    alt_phone: String,
    age: i32,
}

fn customer_validator() -> impl Validator<CreateCustomerRequest> {
    let v = FluentValidator::<CreateCustomerRequest>::new();

    rule_for!(v, c.name)
        .not_empty(None::<String>)
        .min_length(2, None::<String>)
        .max_length(50, None::<String>);

    rule_for!(v, c.email)
        .not_empty(None::<String>)
        .email(None::<String>);

    rule_for!(v, c.age)
        .greater_than_or_equal(18, Some("must be at least 18 years old"))
        .less_than_or_equal(120, None::<String>);

    rule_for!(v, c.phone)
        .not_empty(None::<String>);

    // Cross-property rule: alt_phone must differ from phone
    rule_for!(v, c.alt_phone)
        .not_empty(None::<String>)
        .must(
            |req, val| val != &req.phone,
            "alternative phone cannot be the same as primary phone",
        );

    v.build()
}

fn main() {
    let validator = customer_validator();

    println!("=== Invalid customer ===");
    let invalid = CreateCustomerRequest {
        name: "A".into(),           // too short
        email: "not-an-email".into(),
        phone: "555-1234".into(),
        alt_phone: "555-1234".into(), // same as phone
        age: 15,                    // too young
    };

    let result = validate(&invalid, &validator);
    if !result.is_valid() {
        for error in result.errors() {
            println!("  {}: {}", error.property, error.message);
        }
    }

    println!("\n=== Valid customer ===");
    let valid = CreateCustomerRequest {
        name: "Jane Doe".into(),
        email: "jane@example.com".into(),
        phone: "555-1234".into(),
        alt_phone: "555-5678".into(),
        age: 30,
    };

    let result = validate(&valid, &validator);
    if result.is_valid() {
        println!("  Validation passed.");
    }
}
