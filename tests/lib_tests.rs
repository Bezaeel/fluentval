use fluentval::*;

// ValidationError tests
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

// ValidationResult tests
#[test]
fn test_validation_result_new() {
    let result = ValidationResult::new();
    assert!(result.is_valid());
    assert_eq!(result.errors().len(), 0);
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

// RuleBuilder tests - String rules
#[test]
fn test_rule_builder_not_empty() {
    let rule_fn = RuleBuilder::<String>::for_property("name")
        .not_empty(None::<String>)
        .build();

    assert!(rule_fn(&"".to_string()).len() > 0); // empty string should fail
    assert!(rule_fn(&"   ".to_string()).len() > 0); // whitespace only should fail
    assert!(rule_fn(&"valid".to_string()).is_empty()); // valid string should pass
}

#[test]
fn test_rule_builder_min_length() {
    let rule_fn = RuleBuilder::<String>::for_property("name")
        .min_length(5, None::<String>)
        .build();

    assert!(rule_fn(&"abc".to_string()).len() > 0);
    assert!(rule_fn(&"abcde".to_string()).is_empty());
    assert!(rule_fn(&"abcdef".to_string()).is_empty());
}

#[test]
fn test_rule_builder_max_length() {
    let rule_fn = RuleBuilder::<String>::for_property("name")
        .max_length(5, None::<String>)
        .build();

    assert!(rule_fn(&"abc".to_string()).is_empty());
    assert!(rule_fn(&"abcde".to_string()).is_empty());
    assert!(rule_fn(&"abcdef".to_string()).len() > 0);
}

#[test]
fn test_rule_builder_length() {
    let rule_fn = RuleBuilder::<String>::for_property("name")
        .length(3, 5, None::<String>, None::<String>)
        .build();

    assert!(rule_fn(&"ab".to_string()).len() > 0); // too short
    assert!(rule_fn(&"abc".to_string()).is_empty()); // valid
    assert!(rule_fn(&"abcde".to_string()).is_empty()); // valid
    assert!(rule_fn(&"abcdef".to_string()).len() > 0); // too long
}

#[test]
fn test_rule_builder_email() {
    let rule_fn = RuleBuilder::<String>::for_property("email")
        .email(None::<String>)
        .build();

    assert!(rule_fn(&"invalid".to_string()).len() > 0);
    assert!(rule_fn(&"test@example.com".to_string()).is_empty());
    assert!(rule_fn(&"user.name@domain.co.uk".to_string()).is_empty());
    assert!(rule_fn(&"@example.com".to_string()).len() > 0);
}

// RuleBuilder tests - Numeric rules
#[test]
fn test_rule_builder_greater_than() {
    let rule_fn = RuleBuilder::<i32>::for_property("age")
        .greater_than(18, None::<String>)
        .build();

    assert!(rule_fn(&17).len() > 0);
    assert!(rule_fn(&18).len() > 0);
    assert!(rule_fn(&19).is_empty());
}

#[test]
fn test_rule_builder_greater_than_or_equal() {
    let rule_fn = RuleBuilder::<i32>::for_property("age")
        .greater_than_or_equal(18, None::<String>)
        .build();

    assert!(rule_fn(&17).len() > 0);
    assert!(rule_fn(&18).is_empty());
    assert!(rule_fn(&19).is_empty());
}

#[test]
fn test_rule_builder_less_than() {
    let rule_fn = RuleBuilder::<i32>::for_property("age")
        .less_than(65, None::<String>)
        .build();

    assert!(rule_fn(&64).is_empty());
    assert!(rule_fn(&65).len() > 0);
    assert!(rule_fn(&66).len() > 0);
}

#[test]
fn test_rule_builder_less_than_or_equal() {
    let rule_fn = RuleBuilder::<i32>::for_property("age")
        .less_than_or_equal(65, None::<String>)
        .build();

    assert!(rule_fn(&64).is_empty());
    assert!(rule_fn(&65).is_empty());
    assert!(rule_fn(&66).len() > 0);
}

#[test]
fn test_rule_builder_inclusive_between() {
    let rule_fn = RuleBuilder::<i32>::for_property("age")
        .inclusive_between(18, 65, None::<String>)
        .build();

    assert!(rule_fn(&17).len() > 0);
    assert!(rule_fn(&18).is_empty());
    assert!(rule_fn(&50).is_empty());
    assert!(rule_fn(&65).is_empty());
    assert!(rule_fn(&66).len() > 0);
}

#[test]
fn test_rule_builder_must() {
    let rule_fn = RuleBuilder::<String>::for_property("password")
        .must(|s| s.len() >= 8, "must be at least 8 characters")
        .build();

    assert!(rule_fn(&"short".to_string()).len() > 0);
    assert!(rule_fn(&"longenough".to_string()).is_empty());
}

#[test]
fn test_rule_builder_not_null() {
    let rule_fn = RuleBuilder::<Option<String>>::for_property("value")
        .not_null(None::<String>)
        .build();

    assert!(rule_fn(&None::<String>).len() > 0);
    assert!(rule_fn(&Some("value".to_string())).is_empty());
}

#[test]
fn test_rule_builder_chaining() {
    let rule_fn = RuleBuilder::<String>::for_property("name")
        .not_empty(None::<String>)
        .min_length(3, None::<String>)
        .max_length(10, None::<String>)
        .build();

    assert!(rule_fn(&"".to_string()).len() > 0); // empty
    assert!(rule_fn(&"ab".to_string()).len() > 0); // too short
    assert!(rule_fn(&"abc".to_string()).is_empty()); // valid
    assert!(rule_fn(&"abcdefghij".to_string()).is_empty()); // valid (max)
    assert!(rule_fn(&"abcdefghijk".to_string()).len() > 0); // too long
}

// ValidatorBuilder tests
#[test]
fn test_validator_builder_simple() {
    #[derive(Debug)]
    struct User {
        name: String,
        email: String,
    }

    let validator = ValidatorBuilder::<User>::new()
        .rule_for("name", |u| &u.name,
            RuleBuilder::for_property("name")
                .not_empty(None::<String>)
                .min_length(2, None::<String>))
        .rule_for("email", |u| &u.email,
            RuleBuilder::for_property("email")
                .not_empty(None::<String>)
                .email(None::<String>))
        .build();

    let valid_user = User {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };

    let result = validate(&valid_user, &validator);
    assert!(result.is_valid());

    let invalid_user = User {
        name: "".to_string(),
        email: "invalid".to_string(),
    };

    let result = validate(&invalid_user, &validator);
    assert!(!result.is_valid());
    assert!(result.errors().len() >= 2);
}

#[test]
fn test_validator_builder_numeric() {
    #[derive(Debug)]
    struct Product {
        price: f64,
        quantity: i32,
    }

    let validator = ValidatorBuilder::<Product>::new()
        .rule_for("price", |p| &p.price,
            RuleBuilder::for_property("price")
                .greater_than(0.0, None::<String>)
                .less_than_or_equal(1000.0, None::<String>))
        .rule_for("quantity", |p| &p.quantity,
            RuleBuilder::for_property("quantity")
                .greater_than_or_equal(0, None::<String>)
                .inclusive_between(0, 100, None::<String>))
        .build();

    let valid_product = Product {
        price: 50.0,
        quantity: 10,
    };

    let result = validate(&valid_product, &validator);
    assert!(result.is_valid());

    let invalid_product = Product {
        price: -5.0,
        quantity: 150,
    };

    let result = validate(&invalid_product, &validator);
    assert!(!result.is_valid());
    assert!(result.errors().len() >= 2);
}

#[test]
fn test_validator_builder_multiple_errors() {
    #[derive(Debug)]
    struct User {
        name: String,
        age: i32,
    }

    let validator = ValidatorBuilder::<User>::new()
        .rule_for("name", |u| &u.name,
            RuleBuilder::for_property("name")
                .not_empty(None::<String>)
                .min_length(5, None::<String>)
                .max_length(10, None::<String>))
        .rule_for("age", |u| &u.age,
            RuleBuilder::for_property("age")
                .greater_than_or_equal(18, None::<String>)
                .less_than_or_equal(120, None::<String>))
        .build();

    let invalid_user = User {
        name: "ab".to_string(), // too short
        age: 15, // too young
    };

    let result = validate(&invalid_user, &validator);
    assert!(!result.is_valid());
    
    let errors_by_prop = result.errors_by_property();
    assert!(errors_by_prop.contains_key("name"));
    assert!(errors_by_prop.contains_key("age"));
}

#[test]
fn test_validator_builder_empty_validator() {
    #[derive(Debug)]
    struct EmptyStruct {
        #[allow(dead_code)]
        value: String,
    }

    let validator = ValidatorBuilder::<EmptyStruct>::new().build();
    let instance = EmptyStruct {
        value: "anything".to_string(),
    };

    let result = validate(&instance, &validator);
    assert!(result.is_valid());
}

#[test]
fn test_rule_builder_custom_rule() {
    let rule_fn = RuleBuilder::<String>::for_property("value")
        .rule(|v| {
            if v.contains("forbidden") {
                Some("contains forbidden word".to_string())
            } else {
                None
            }
        })
        .build();

    assert!(rule_fn(&"forbidden word".to_string()).len() > 0);
    assert!(rule_fn(&"allowed word".to_string()).is_empty());
}

#[test]
fn test_numeric_trait_implementations() {
    assert_eq!(5i8.to_f64(), 5.0);
    assert_eq!(10i32.to_f64(), 10.0);
    assert_eq!(20u32.to_f64(), 20.0);
    // f32 to f64 conversion may have slight precision differences
    assert!((3.14f32.to_f64() - 3.14).abs() < 0.0001);
    assert_eq!(2.71f64.to_f64(), 2.71);
}

#[test]
fn test_option_like_trait() {
    let some: Option<String> = Some("value".to_string());
    let none: Option<String> = None;

    assert!(!some.is_none());
    assert!(none.is_none());
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
fn test_rule_builder_custom_messages() {
    // not_empty with custom message
    let rule_fn = RuleBuilder::<String>::for_property("name")
        .not_empty(Some("custom not empty"))
        .build();
    assert_eq!(rule_fn(&"".to_string())[0].message, "custom not empty");

    // not_null with custom message
    let rule_fn = RuleBuilder::<Option<String>>::for_property("val")
        .not_null(Some("custom not null"))
        .build();
    assert_eq!(rule_fn(&None::<String>)[0].message, "custom not null");

    // min_length with custom message
    let rule_fn = RuleBuilder::<String>::for_property("name")
        .min_length(5, Some("custom min length"))
        .build();
    assert_eq!(rule_fn(&"abc".to_string())[0].message, "custom min length");

    // max_length with custom message
    let rule_fn = RuleBuilder::<String>::for_property("name")
        .max_length(3, Some("custom max length"))
        .build();
    assert_eq!(rule_fn(&"abcdef".to_string())[0].message, "custom max length");

    // email with custom message
    let rule_fn = RuleBuilder::<String>::for_property("email")
        .email(Some("custom email error"))
        .build();
    assert_eq!(rule_fn(&"invalid".to_string())[0].message, "custom email error");

    // greater_than with custom message
    let rule_fn = RuleBuilder::<i32>::for_property("age")
        .greater_than(18, Some("custom greater than"))
        .build();
    assert_eq!(rule_fn(&10)[0].message, "custom greater than");

    // greater_than_or_equal with custom message
    let rule_fn = RuleBuilder::<i32>::for_property("age")
        .greater_than_or_equal(18, Some("custom gte"))
        .build();
    assert_eq!(rule_fn(&10)[0].message, "custom gte");

    // less_than with custom message
    let rule_fn = RuleBuilder::<i32>::for_property("age")
        .less_than(65, Some("custom less than"))
        .build();
    assert_eq!(rule_fn(&100)[0].message, "custom less than");

    // less_than_or_equal with custom message
    let rule_fn = RuleBuilder::<i32>::for_property("age")
        .less_than_or_equal(65, Some("custom lte"))
        .build();
    assert_eq!(rule_fn(&100)[0].message, "custom lte");

    // inclusive_between with custom message
    let rule_fn = RuleBuilder::<i32>::for_property("score")
        .inclusive_between(0, 100, Some("custom between"))
        .build();
    assert_eq!(rule_fn(&150)[0].message, "custom between");
}

#[test]
fn test_validation_result_default() {
    let result = ValidationResult::default();
    assert!(result.is_valid());
}

#[test]
fn test_validator_builder_default() {
    let builder = ValidatorBuilder::<String>::default();
    let validator = builder.build();
    let result = validate(&"test".to_string(), &validator);
    assert!(result.is_valid());
}

#[test]
fn test_validator_builder_must_with_object() {
    #[derive(Debug)]
    struct Command {
        country_iso_code: String,
        phone_number: String,
        alt_phone_number: String,
    }

    // Helper function to validate phone number
    fn is_valid_phone_number_for_country(phone: &str, country_code: &str) -> bool {
        match country_code {
            "US" => phone.len() == 10 && phone.chars().all(|c| c.is_ascii_digit()),
            "UK" => phone.len() == 11 && phone.starts_with('0'),
            _ => phone.len() >= 8 && phone.len() <= 15,
        }
    }

    let validator = ValidatorBuilder::<Command>::new()
        .rule_for("phoneNumber", |c| &c.phone_number,
            RuleBuilder::for_property("phoneNumber")
                .not_empty(None::<String>))
        .must("phoneNumber", |c| &c.phone_number,
            |command, phone_number| is_valid_phone_number_for_country(phone_number, &command.country_iso_code),
            "Phone number is not valid for the specified country")
        .must("altPhoneNumber", |c| &c.alt_phone_number,
            |command, alt_phone| alt_phone != &command.phone_number,
            "Alternative phone number must be different from primary phone number")
        .build();

    // Test invalid: phone number doesn't match country
    let invalid_command = Command {
        country_iso_code: "US".to_string(),
        phone_number: "123".to_string(),  // Too short for US
        alt_phone_number: "9876543210".to_string(),
    };

    let result = validate(&invalid_command, &validator);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| e.property == "phoneNumber"));

    // Test invalid: alt phone same as primary
    let invalid_command2 = Command {
        country_iso_code: "US".to_string(),
        phone_number: "1234567890".to_string(),
        alt_phone_number: "1234567890".to_string(),  // Same as primary
    };

    let result = validate(&invalid_command2, &validator);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| e.property == "altPhoneNumber"));

    // Test valid
    let valid_command = Command {
        country_iso_code: "US".to_string(),
        phone_number: "1234567890".to_string(),  // Valid US phone
        alt_phone_number: "9876543210".to_string(),  // Valid and different
    };

    let result = validate(&valid_command, &validator);
    assert!(result.is_valid());
}

#[test]
fn test_validator_builder_must_with_country_validation() {
    #[derive(Debug)]
    struct Command {
        country: String,
        tax_number: String,
        country_iso_code: String,
    }

    // Simulate allowed countries
    struct Countries;
    impl Countries {
        fn allowed_countries() -> Vec<&'static str> {
            vec!["US", "UK", "CA", "AU"]
        }
    }

    // Helper function to validate tax number
    fn is_valid_tax_number(tax_number: &str, country_code: &str) -> bool {
        match country_code {
            "US" => tax_number.len() == 9 && tax_number.chars().all(|c| c.is_ascii_digit()),
            "UK" => tax_number.len() == 10 && tax_number.starts_with("GB"),
            _ => tax_number.len() >= 8 && tax_number.len() <= 15,
        }
    }

    let validator = ValidatorBuilder::<Command>::new()
        // Example 1: Validate country ignoring the object (use _ for object parameter)
        .must("country", |c| &c.country,
            |_, country| Countries::allowed_countries().contains(&country.as_str()),
            "Country is not in the allowed list")
        // Example 2: Validate tax number using both object and property value
        .must("taxNumber", |c| &c.tax_number,
            |command, tax_number| is_valid_tax_number(tax_number, &command.country_iso_code),
            "Tax number is not valid for the specified country")
        .build();

    // Test invalid: country not in allowed list
    let invalid_command = Command {
        country: "FR".to_string(),  // Not in allowed list
        tax_number: "123456789".to_string(),
        country_iso_code: "US".to_string(),
    };

    let result = validate(&invalid_command, &validator);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| e.property == "country"));

    // Test invalid: tax number doesn't match country
    let invalid_command2 = Command {
        country: "US".to_string(),
        tax_number: "123".to_string(),  // Too short for US
        country_iso_code: "US".to_string(),
    };

    let result = validate(&invalid_command2, &validator);
    assert!(!result.is_valid());
    assert!(result.errors().iter().any(|e| e.property == "taxNumber"));

    // Test valid
    let valid_command = Command {
        country: "US".to_string(),  // In allowed list
        tax_number: "123456789".to_string(),  // Valid US tax number
        country_iso_code: "US".to_string(),
    };

    let result = validate(&valid_command, &validator);
    assert!(result.is_valid());
}

