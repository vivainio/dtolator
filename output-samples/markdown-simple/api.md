# Sample API

**Version:** 1.0.0

A sample API to demonstrate dtolator

## Users

<details>
<summary><code>GET</code> /users — <strong>List All Users</strong></summary>

Retrieve a list of all users in the system

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

</details>

<details>
<summary><code>POST</code> /users — <strong>Create New User</strong></summary>

Create a new user account

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

</details>

<details>
<summary><code>GET</code> /users/{userId} — <strong>Get User By ID</strong></summary>

Retrieve a specific user by their ID

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Responses:**

- **200**: User found → `User`
- **404**: User not found

</details>

<details>
<summary><code>PUT</code> /users/{userId} — <strong>Update User Profile</strong></summary>

Update an existing user's information

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Request body:** `CreateUserRequest`

**Responses:**

- **200**: User updated successfully → `ApiResponse`

</details>

<details>
<summary><code>DELETE</code> /users/{userId} — <strong>Delete User Account</strong></summary>

Permanently delete a user account

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Responses:**

- **200**: User deleted successfully → `ApiResponse`

</details>

<details>
<summary><code>POST</code> /users/{userId}/activate — <strong>Activate User Account</strong></summary>

Activate a user's account status

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Responses:**

- **200**: User activated successfully → `ApiResponse`

</details>

<details>
<summary><code>POST</code> /users/{userId}/deactivate — <strong>Deactivate User Account</strong></summary>

Deactivate a user's account status

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `integer` | yes |  |

**Responses:**

- **200**: User deactivated successfully → `ApiResponse`

</details>

