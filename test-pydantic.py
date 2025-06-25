# Generated Pydantic models from OpenAPI schema
# Do not modify this file manually

from datetime import date, datetime
from enum import Enum
from typing import Any, Dict, List, Literal, Optional, Union
from uuid import UUID

from pydantic import BaseModel, EmailStr, Field, HttpUrl


class User(BaseModel):
    id: int
    email: EmailStr
    name: str = Field(min_length=1, max_length=100)
    age: Optional[int] = Field(None, ge=0, le=150)
    isActive: Optional[bool] = None
    tags: Optional[List[str]] = None
    status: Optional[Literal["active", "inactive", "pending"]] = None
    profile: Optional[UserProfile] = None
    address: Optional[Address] = None


class UserProfile(BaseModel):
    firstName: str = Field(min_length=1, max_length=50)
    lastName: str = Field(min_length=1, max_length=50)
    phoneNumber: Optional[str] = Field(None, regex=r"^\+?[1-9]\d{1,14}$")
    avatar: Optional[HttpUrl] = None
    bio: Optional[str] = Field(None, max_length=500)


class Address(BaseModel):
    street: str = Field(min_length=1, max_length=100)
    city: str = Field(min_length=1, max_length=50)
    state: Optional[str] = Field(None, max_length=50)
    country: str = Field(
        regex=r"^[A-Z]{2}$", description="ISO 3166-1 alpha-2 country code"
    )
    postalCode: Optional[str] = Field(None, min_length=3, max_length=10)


class CreateUserRequest(BaseModel):
    email: EmailStr
    name: str = Field(min_length=1, max_length=100)
    age: Optional[int] = Field(None, ge=0, le=150)
    profile: UserProfile
    address: Optional[Address] = None


class ApiResponse(BaseModel):
    success: bool
    message: Optional[str] = None
    data: Optional[User] = None
