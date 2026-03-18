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

### General Notes
- The codebase prioritizes explicit, verbose code for learning purposes
- Avoid implicit returns and shorthand syntax
- Code clarity and readability are more important than conciseness
- Suppress warnings when compiling with cargo build for easier compilation error debugging
