# Sample API

## Users

`GET` /users → `User[]`

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

`POST` /users `CreateUserRequest` → `ApiResponse`

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

`GET` /users/{userId} → `User`

`PUT` /users/{userId} `CreateUserRequest` → `ApiResponse`

`DELETE` /users/{userId} → `ApiResponse`

`POST` /users/{userId}/activate → `ApiResponse`

`POST` /users/{userId}/deactivate → `ApiResponse`

