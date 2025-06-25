# Generated Python TypedDict definitions from OpenAPI schema
# Do not modify this file manually

from datetime import date, datetime
from enum import Enum
from typing import Any, Dict, List, Literal, Optional, Union
from typing_extensions import TypedDict
from uuid import UUID

class UserRequired(TypedDict):
    id: int
    email: str
    name: str

class User(UserRequired, total=False):
    age: int
    isActive: bool
    tags: List[str]
    status: Literal["active", "inactive", "pending"]
    profile: UserProfile
    address: Address

class UserProfileRequired(TypedDict):
    firstName: str
    lastName: str

class UserProfile(UserProfileRequired, total=False):
    phoneNumber: Optional[str]
    avatar: Optional[str]
    bio: Optional[str]

class AddressRequired(TypedDict):
    street: str
    city: str
    country: str

class Address(AddressRequired, total=False):
    state: Optional[str]
    postalCode: Optional[str]

class CreateUserRequestRequired(TypedDict):
    email: str
    name: str
    profile: UserProfile

class CreateUserRequest(CreateUserRequestRequired, total=False):
    age: Optional[int]
    address: Address

class ApiResponseRequired(TypedDict):
    success: bool

class ApiResponse(ApiResponseRequired, total=False):
    message: str
    data: User


