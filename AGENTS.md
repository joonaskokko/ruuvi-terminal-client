# AI Agent Instructions

## Code Style Guidelines

### Conditional Statements
- **All `if` statements MUST use parentheses** around the condition
- **Good:** `if (condition) { ... }`
- **Bad:** `if condition { ... }`

### Function Returns
- **All functions MUST have explicit `return` statements** at the end
- **Good:** 
	```rust
	fn example() -> Result<String, Error> {
			// code...
			return Ok(data);
	}
	```
- **Bad:**
	```rust
	fn example() -> Result<String, Error> {
			// code...
			Ok(data)
	}
	```

### Variable naming
- **All variables should have full words in them and not shortened. Exceptions: "min" and "max" can be used.**
- **Good:**
	```rust
	let temperature_string = format!("{:+.2}°C", tag.temperature.current);
	```
- **Bad:**
	```rust
	let tmp_str = format!("{:+.2}°C", tag.tmp.current);
	```

### General Notes
- Use tabs instead of spaces
- The codebase prioritizes explicit, verbose code for learning purposes
- Avoid implicit returns and shorthand syntax
- Code clarity and readability are more important than conciseness
- Suppress warnings when compiling with cargo build for easier compilation error debugging
