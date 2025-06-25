// Generated C# classes from OpenAPI schema
// Do not modify this file manually

using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace GeneratedApiModels
{
    public class User
    {
        [JsonPropertyName("id")]
        public long Id { get; set; }
        [JsonPropertyName("email")]
        public string Email { get; set; }
        [JsonPropertyName("name")]
        public string Name { get; set; }
        [JsonPropertyName("age")]
        public int? Age { get; set; }
        [JsonPropertyName("isActive")]
        public bool? IsActive { get; set; }
        [JsonPropertyName("tags")]
        public List<string> Tags { get; set; }
        [JsonPropertyName("status")]
        public string Status { get; set; }
        [JsonPropertyName("profile")]
        public UserProfile Profile { get; set; }
        [JsonPropertyName("address")]
        public Address Address { get; set; }
    }

    public class UserProfile
    {
        [JsonPropertyName("firstName")]
        public string FirstName { get; set; }
        [JsonPropertyName("lastName")]
        public string LastName { get; set; }
        [JsonPropertyName("phoneNumber")]
        public string PhoneNumber { get; set; }
        [JsonPropertyName("avatar")]
        public string Avatar { get; set; }
        [JsonPropertyName("bio")]
        public string Bio { get; set; }
    }

    public class Address
    {
        [JsonPropertyName("street")]
        public string Street { get; set; }
        [JsonPropertyName("city")]
        public string City { get; set; }
        [JsonPropertyName("state")]
        public string State { get; set; }
        [JsonPropertyName("country")]
        public string Country { get; set; }
        [JsonPropertyName("postalCode")]
        public string PostalCode { get; set; }
    }

    public class CreateUserRequest
    {
        [JsonPropertyName("email")]
        public string Email { get; set; }
        [JsonPropertyName("name")]
        public string Name { get; set; }
        [JsonPropertyName("age")]
        public int? Age { get; set; }
        [JsonPropertyName("profile")]
        public UserProfile Profile { get; set; }
        [JsonPropertyName("address")]
        public Address Address { get; set; }
    }

    public class ApiResponse
    {
        [JsonPropertyName("success")]
        public bool Success { get; set; }
        [JsonPropertyName("message")]
        public string Message { get; set; }
        [JsonPropertyName("data")]
        public User Data { get; set; }
    }

}
