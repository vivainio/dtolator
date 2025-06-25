# Pydantic Sample Output

This directory contains sample Pydantic BaseModel classes generated from OpenAPI schemas using the `--pydantic` flag.

## Generated Files

### models.py
Generated from `simple-sample.json` using:
```bash
dtolator -i simple-sample.json --pydantic --pretty
```

Contains Pydantic BaseModel classes with:
- Type-safe field definitions
- Validation constraints (min/max length, regex patterns, etc.)
- Email validation using `EmailStr`
- Enum support with `Literal` types
- Optional fields with proper `Optional[]` typing
- Field descriptions and constraints using `Field()`

## Features Demonstrated

- **Email Validation**: `email: EmailStr` automatically validates email format
- **String Constraints**: `name: str = Field(min_length=1, max_length=100)` 
- **Numeric Validation**: `age: Optional[int] = Field(None, ge=0, le=150)`
- **Enum Values**: `status: Optional[Literal["active", "inactive", "pending"]] = None`
- **Optional Fields**: Proper handling of nullable and optional fields
- **Nested Models**: Support for complex object relationships

## Usage in Python

```python
from models import User, CreateUserRequest, ApiResponse

# Create a user instance with validation
user = User(
    id=123,
    email="john@example.com",
    name="John Doe",
    age=30,
    status="active"
)

# Validation happens automatically
try:
    invalid_user = User(
        id=123,
        email="invalid-email",  # Will raise ValidationError
        name="",               # Will raise ValidationError (min_length=1)
        age=200               # Will raise ValidationError (le=150)
    )
except ValidationError as e:
    print(f"Validation failed: {e}")

# JSON serialization/deserialization
user_json = user.model_dump_json()
user_from_json = User.model_validate_json(user_json) 