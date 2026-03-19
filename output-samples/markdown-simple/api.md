# Sample API

**Version:** 1.0.0

A sample API to demonstrate dtolator

## Users

#### `GET` /users

**List All Users** — Retrieve a list of all users in the system

**Responses:**

- **200**: Successful response → `User[]`

```typescript
// Represents a user in the system.
//
// A user can be in one of the following states:
// - **active**: The user can log in and use the system.
// - **inactive**: The user account is disabled.
// - **pending**: The user has registered but not yet confirmed their email.
//
// See also: `UserProfile` for extended profile information.
type User {
  id: integer  // int64
  email: string  // email, The user's primary email address. Must be unique across the system. Used for login and notifications.
  name: string
  age?: integer
  isActive?: boolean
  tags?: string[]
  status?: "active" | "inactive" | "pending"
  profile?: UserProfile
  address?: Address
}

// A physical mailing address.
//
// All addresses must include at least `street`, `city`, and `country`.
// The `country` field uses ISO 3166-1 alpha-2 codes (e.g. `US`, `FI`, `DE`).
type Address {
  street: string
  city: string
  state?: string | null
  country: string  // ISO 3166-1 alpha-2 country code
  postalCode?: string | null
}

type UserProfile {
  firstName: string
  lastName: string
  phoneNumber?: string | null
  avatar?: string | null  // uri
  bio?: string | null
}

```

#### `POST` /users

**Create New User** — Create a new user account

**Request body:** `CreateUserRequest`

**Responses:**

- **201**: User created successfully → `ApiResponse`

```typescript
type CreateUserRequest {
  email: string  // email
  name: string
  age?: integer | null
  profile: UserProfile
  address?: Address
}

type ApiResponse {
  success: boolean
  message?: string
  data?: User
}

```

#### `GET` /users/{userId}

**Get User By ID** — Retrieve a specific user by their ID

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Responses:**

- **200**: User found → `User`
- **404**: User not found

#### `PUT` /users/{userId}

**Update User Profile** — Update an existing user's information

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Request body:** `CreateUserRequest`

**Responses:**

- **200**: User updated successfully → `ApiResponse`

#### `DELETE` /users/{userId}

**Delete User Account** — Permanently delete a user account

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Responses:**

- **200**: User deleted successfully → `ApiResponse`

#### `POST` /users/{userId}/activate

**Activate User Account** — Activate a user's account status

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Responses:**

- **200**: User activated successfully → `ApiResponse`

#### `POST` /users/{userId}/deactivate

**Deactivate User Account** — Deactivate a user's account status

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Responses:**

- **200**: User deactivated successfully → `ApiResponse`

