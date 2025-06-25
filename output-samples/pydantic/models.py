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

class CreateUserRequest(BaseModel):
    email: EmailStr
    name: str = Field(min_length=1, max_length=100)
    age: Optional[int] = Field(None, ge=0, le=150)

class ApiResponse(BaseModel):
    success: bool
    message: Optional[str] = None
    data: Optional[User] = None


