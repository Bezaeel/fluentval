use fluentval::*;

// Example struct to validate
#[derive(Debug)]
struct User {
    name: String,
    email: String,
    age: i32,
    password: String,
}

fn main() {
    // Create a validator using the fluent API
    // You can pass None::<String> for default messages, or Some("custom message") for custom messages
    let validator = ValidatorBuilder::<User>::new()
        .rule_for("name", |u| &u.name, 
            RuleBuilder::for_property("name")
                .not_empty(None::<String>)  // Use default message
                .min_length(2, None::<String>)  // Use default message
                .max_length(50, None::<String>))  // Use default message
        .rule_for("email", |u| &u.email,
            RuleBuilder::for_property("email")
                .not_empty(None::<String>)  // Use default message
                .email(None::<String>))  // Use default message
        .rule_for("age", |u| &u.age,
            RuleBuilder::for_property("age")
                .greater_than_or_equal(18, None::<String>)  // Use default message
                .less_than_or_equal(120, None::<String>))  // Use default message
        .rule_for("password", |u| &u.password,
            RuleBuilder::for_property("password")
                .not_empty(None::<String>)  // Use default message
                .min_length(8, None::<String>)  // Use default message
                .must(|p: &String| p.chars().any(|c| c.is_ascii_uppercase()), "must contain at least one uppercase letter")
                .must(|p: &String| p.chars().any(|c| c.is_ascii_lowercase()), "must contain at least one lowercase letter")
                .must(|p: &String| p.chars().any(|c| c.is_ascii_digit()), "must contain at least one digit"))
        .build();

    // Test with invalid data
    println!("=== Testing with invalid data ===");
    let invalid_user = User {
        name: "A".to_string(), // Too short
        email: "invalid-email".to_string(), // Invalid email
        age: 15, // Too young
        password: "weak".to_string(), // Too weak
    };

    let result = validate(&invalid_user, &validator);
    
    if !result.is_valid() {
        println!("Validation failed with {} errors:", result.errors().len());
        for error in result.errors() {
            println!("  - {}", error);
        }
        
        println!("\nErrors by property:");
        for (property, messages) in result.errors_by_property() {
            println!("  {}:", property);
            for msg in messages {
                println!("    - {}", msg);
            }
        }
    }

    // Test with valid data
    println!("\n=== Testing with valid data ===");
    let valid_user = User {
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        age: 25,
        password: "SecurePass123".to_string(),
    };

    let result = validate(&valid_user, &validator);
    
    if result.is_valid() {
        println!("✓ Validation passed!");
    } else {
        println!("Validation failed:");
        for error in result.errors() {
            println!("  - {}", error);
        }
    }

    // Example with numeric validation
    println!("\n=== Example: Numeric validation ===");
    #[derive(Debug)]
    struct Product {
        price: f64,
        quantity: i32,
    }

    let product_validator = ValidatorBuilder::<Product>::new()
        .rule_for("price", |p| &p.price,
            RuleBuilder::for_property("price")
                .greater_than(0.0, None::<String>)  // Use default message
                .less_than_or_equal(10000.0, None::<String>))  // Use default message
        .rule_for("quantity", |p| &p.quantity,
            RuleBuilder::for_property("quantity")
                .greater_than_or_equal(0, None::<String>)  // Use default message
                .inclusive_between(0, 1000, None::<String>))  // Use default message
        .build();

    let product = Product {
        price: -5.0, // Invalid: negative price
        quantity: 1500, // Invalid: exceeds max
    };

    let result = validate(&product, &product_validator);
    if !result.is_valid() {
        println!("Product validation failed:");
        for error in result.errors() {
            println!("  - {}", error);
        }
    }

    // Example with custom error messages
    println!("\n=== Example: Custom error messages ===");
    #[derive(Debug)]
    struct Order {
        quantity: i32,
        discount: f64,
    }

    let order_validator = ValidatorBuilder::<Order>::new()
        .rule_for("quantity", |o| &o.quantity,
            RuleBuilder::for_property("quantity")
                .greater_than_or_equal(1, Some("Quantity must be at least 1".to_string()))
                .less_than_or_equal(100, Some("Quantity cannot exceed 100 items".to_string())))
        .rule_for("discount", |o| &o.discount,
            RuleBuilder::for_property("discount")
                .greater_than_or_equal(0.0, Some("Discount cannot be negative".to_string()))
                .less_than_or_equal(1.0, Some("Discount cannot exceed 100%".to_string())))
        .build();

    let invalid_order = Order {
        quantity: 0,  // Invalid
        discount: 1.5,  // Invalid: exceeds 100%
    };

    let result = validate(&invalid_order, &order_validator);
    if !result.is_valid() {
        println!("Order validation failed with custom messages:");
        for error in result.errors() {
            println!("  - {}", error);
        }
    }

    // Example with cross-property validation using .must()
    println!("\n=== Example: Cross-property validation with .must() ===");
    #[derive(Debug)]
    struct Command {
        country_iso_code: String,
        phone_number: String,
        alt_phone_number: String,
    }

    // Helper function to validate phone number (simplified example)
    fn is_valid_phone_number_for_country(phone: &str, country_code: &str) -> bool {
        match country_code {
            "US" => phone.len() == 10 && phone.chars().all(|c| c.is_ascii_digit()),
            "UK" => phone.len() == 11 && phone.starts_with('0'),
            _ => phone.len() >= 8 && phone.len() <= 15,
        }
    }

    let command_validator = ValidatorBuilder::<Command>::new()
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

    let invalid_command = Command {
        country_iso_code: "US".to_string(),
        phone_number: "123".to_string(),  // Invalid: too short for US
        alt_phone_number: "123".to_string(),  // Invalid: same as primary
    };

    let result = validate(&invalid_command, &command_validator);
    if !result.is_valid() {
        println!("Command validation failed:");
        for error in result.errors() {
            println!("  - {}", error);
        }
    }

    let valid_command = Command {
        country_iso_code: "US".to_string(),
        phone_number: "1234567890".to_string(),  // Valid US phone
        alt_phone_number: "9876543210".to_string(),  // Valid and different
    };

    let result = validate(&valid_command, &command_validator);
    if result.is_valid() {
        println!("✓ Command validation passed!");
    } else {
        println!("Command validation failed:");
        for error in result.errors() {
            println!("  - {}", error);
        }
    }

    // Example with country and tax number validation
    println!("\n=== Example: Country and Tax Number validation ===");
    #[derive(Debug)]
    struct TaxCommand {
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

    let tax_validator = ValidatorBuilder::<TaxCommand>::new()
        // Example 1: Validate country ignoring the object (use _ for object parameter)
        .must("country", |c| &c.country,
            |_, country| Countries::allowed_countries().contains(&country.as_str()),
            "Country is not in the allowed list")
        // Example 2: Validate tax number using both object and property value
        .must("taxNumber", |c| &c.tax_number,
            |command, tax_number| is_valid_tax_number(tax_number, &command.country_iso_code),
            "Tax number is not valid for the specified country")
        .build();

    let invalid_tax_command = TaxCommand {
        country: "FR".to_string(),  // Not in allowed list
        tax_number: "123".to_string(),  // Invalid for US
        country_iso_code: "US".to_string(),
    };

    let result = validate(&invalid_tax_command, &tax_validator);
    if !result.is_valid() {
        println!("Tax command validation failed:");
        for error in result.errors() {
            println!("  - {}", error);
        }
    }

    let valid_tax_command = TaxCommand {
        country: "US".to_string(),  // In allowed list
        tax_number: "123456789".to_string(),  // Valid US tax number
        country_iso_code: "US".to_string(),
    };

    let result = validate(&valid_tax_command, &tax_validator);
    if result.is_valid() {
        println!("✓ Tax command validation passed!");
    } else {
        println!("Tax command validation failed:");
        for error in result.errors() {
            println!("  - {}", error);
        }
    }
}
