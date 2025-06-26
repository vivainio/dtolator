// Generated C# records from OpenAPI schema
// Do not modify this file manually

using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace GeneratedApiModels;

public record User
{
    [JsonPropertyName("id")]
    public required Guid Id { get; set; }
    [JsonPropertyName("email")]
    public required string Email { get; set; }
    [JsonPropertyName("profile")]
    public required UserProfile Profile { get; set; }
    [JsonPropertyName("preferences")]
    public UserPreferences? Preferences { get; set; }
    [JsonPropertyName("createdAt")]
    public DateTime? CreatedAt { get; set; }
    [JsonPropertyName("updatedAt")]
    public DateTime? UpdatedAt { get; set; }
    [JsonPropertyName("isActive")]
    public bool? IsActive { get; set; }
    [JsonPropertyName("roles")]
    public List<UserRole> Roles { get; set; }
}

public record UserProfile
{
    [JsonPropertyName("firstName")]
    public required string FirstName { get; set; }
    [JsonPropertyName("lastName")]
    public required string LastName { get; set; }
    [JsonPropertyName("dateOfBirth")]
    public DateOnly? DateOfBirth { get; set; }
    [JsonPropertyName("phoneNumber")]
    public string? PhoneNumber { get; set; }
    [JsonPropertyName("avatar")]
    public ImageUrl? Avatar { get; set; }
    [JsonPropertyName("address")]
    public Address? Address { get; set; }
}

public record UserPreferences
{
    [JsonPropertyName("language")]
    public string? Language { get; set; }
    [JsonPropertyName("currency")]
    public string? Currency { get; set; }
    [JsonPropertyName("notifications")]
    public NotificationSettings? Notifications { get; set; }
    [JsonPropertyName("theme")]
    public string? Theme { get; set; }
}

public record NotificationSettings
{
    [JsonPropertyName("email")]
    public bool? Email { get; set; }
    [JsonPropertyName("push")]
    public bool? Push { get; set; }
    [JsonPropertyName("sms")]
    public bool? Sms { get; set; }
    [JsonPropertyName("marketing")]
    public bool? Marketing { get; set; }
}

public enum UserRole
{
    [JsonPropertyName("customer")]
    Customer,
    [JsonPropertyName("admin")]
    Admin,
    [JsonPropertyName("moderator")]
    Moderator,
    [JsonPropertyName("vendor")]
    Vendor
}

public record Address
{
    [JsonPropertyName("street")]
    public required string Street { get; set; }
    [JsonPropertyName("street2")]
    public string? Street2 { get; set; }
    [JsonPropertyName("city")]
    public required string City { get; set; }
    [JsonPropertyName("state")]
    public string? State { get; set; }
    [JsonPropertyName("country")]
    public required string Country { get; set; }
    [JsonPropertyName("postalCode")]
    public required string PostalCode { get; set; }
}

public record ImageUrl
{
    [JsonPropertyName("url")]
    public required string Url { get; set; }
    [JsonPropertyName("alt")]
    public string? Alt { get; set; }
    [JsonPropertyName("width")]
    public int? Width { get; set; }
    [JsonPropertyName("height")]
    public int? Height { get; set; }
}

public record Product
{
    [JsonPropertyName("id")]
    public required Guid Id { get; set; }
    [JsonPropertyName("name")]
    public required string Name { get; set; }
    [JsonPropertyName("description")]
    public string? Description { get; set; }
    [JsonPropertyName("price")]
    public required Price Price { get; set; }
    [JsonPropertyName("category")]
    public required ProductCategory Category { get; set; }
    [JsonPropertyName("tags")]
    public List<string> Tags { get; set; }
    [JsonPropertyName("images")]
    public List<ImageUrl> Images { get; set; }
    [JsonPropertyName("inventory")]
    public Inventory? Inventory { get; set; }
    [JsonPropertyName("specifications")]
    public Dictionary<string, object>? Specifications { get; set; }
    [JsonPropertyName("isActive")]
    public bool? IsActive { get; set; }
    [JsonPropertyName("createdAt")]
    public DateTime? CreatedAt { get; set; }
}

public record Price
{
    [JsonPropertyName("amount")]
    public required double Amount { get; set; }
    [JsonPropertyName("currency")]
    public required string Currency { get; set; }
    [JsonPropertyName("originalAmount")]
    public double? OriginalAmount { get; set; }
}

public enum ProductCategory
{
    [JsonPropertyName("electronics")]
    Electronics,
    [JsonPropertyName("clothing")]
    Clothing,
    [JsonPropertyName("home")]
    Home,
    [JsonPropertyName("books")]
    Books,
    [JsonPropertyName("sports")]
    Sports,
    [JsonPropertyName("beauty")]
    Beauty,
    [JsonPropertyName("automotive")]
    Automotive
}

public record Inventory
{
    [JsonPropertyName("quantity")]
    public required int Quantity { get; set; }
    [JsonPropertyName("status")]
    public required string Status { get; set; }
    [JsonPropertyName("lowStockThreshold")]
    public int? LowStockThreshold { get; set; }
}

public record Order
{
    [JsonPropertyName("id")]
    public required Guid Id { get; set; }
    [JsonPropertyName("userId")]
    public required Guid UserId { get; set; }
    [JsonPropertyName("items")]
    public required List<OrderItem> Items { get; set; }
    [JsonPropertyName("total")]
    public required Price Total { get; set; }
    [JsonPropertyName("status")]
    public required OrderStatus Status { get; set; }
    [JsonPropertyName("shippingAddress")]
    public Address? ShippingAddress { get; set; }
    [JsonPropertyName("billingAddress")]
    public Address? BillingAddress { get; set; }
    [JsonPropertyName("paymentMethod")]
    public PaymentMethod? PaymentMethod { get; set; }
    [JsonPropertyName("orderDate")]
    public DateTime? OrderDate { get; set; }
    [JsonPropertyName("estimatedDelivery")]
    public DateOnly? EstimatedDelivery { get; set; }
    [JsonPropertyName("trackingNumber")]
    public string? TrackingNumber { get; set; }
}

public record OrderItem
{
    [JsonPropertyName("productId")]
    public required Guid ProductId { get; set; }
    [JsonPropertyName("quantity")]
    public required int Quantity { get; set; }
    [JsonPropertyName("price")]
    public required Price Price { get; set; }
    [JsonPropertyName("productSnapshot")]
    public Product? ProductSnapshot { get; set; }
}

public enum OrderStatus
{
    [JsonPropertyName("pending")]
    Pending,
    [JsonPropertyName("confirmed")]
    Confirmed,
    [JsonPropertyName("processing")]
    Processing,
    [JsonPropertyName("shipped")]
    Shipped,
    [JsonPropertyName("delivered")]
    Delivered,
    [JsonPropertyName("cancelled")]
    Cancelled,
    [JsonPropertyName("refunded")]
    Refunded
}

public record PaymentMethod
{
    [JsonPropertyName("type")]
    public required string Type { get; set; }
    [JsonPropertyName("last4")]
    public string? Last4 { get; set; }
    [JsonPropertyName("brand")]
    public string? Brand { get; set; }
}

public record CreateUserRequest
{
    [JsonPropertyName("email")]
    public required string Email { get; set; }
    [JsonPropertyName("password")]
    public required string Password { get; set; }
    [JsonPropertyName("profile")]
    public required UserProfile Profile { get; set; }
    [JsonPropertyName("preferences")]
    public UserPreferences? Preferences { get; set; }
}

public record CreateOrderRequest
{
    [JsonPropertyName("items")]
    public required List<Dictionary<string, object>> Items { get; set; }
    [JsonPropertyName("shippingAddress")]
    public required Address ShippingAddress { get; set; }
    [JsonPropertyName("billingAddress")]
    public Address? BillingAddress { get; set; }
    [JsonPropertyName("paymentMethod")]
    public PaymentMethod? PaymentMethod { get; set; }
}

public record UserListResponse
{
    [JsonPropertyName("data")]
    public required List<User> Data { get; set; }
    [JsonPropertyName("pagination")]
    public required PaginationInfo Pagination { get; set; }
}

public record ProductListResponse
{
    [JsonPropertyName("data")]
    public required List<Product> Data { get; set; }
    [JsonPropertyName("pagination")]
    public required PaginationInfo Pagination { get; set; }
    [JsonPropertyName("filters")]
    public Dictionary<string, object>? Filters { get; set; }
}

public record PaginationInfo
{
    [JsonPropertyName("page")]
    public required int Page { get; set; }
    [JsonPropertyName("limit")]
    public required int Limit { get; set; }
    [JsonPropertyName("total")]
    public required int Total { get; set; }
    [JsonPropertyName("totalPages")]
    public required int TotalPages { get; set; }
    [JsonPropertyName("hasNext")]
    public bool? HasNext { get; set; }
    [JsonPropertyName("hasPrev")]
    public bool? HasPrev { get; set; }
}

public record ErrorResponse
{
    [JsonPropertyName("error")]
    public required Dictionary<string, object> Error { get; set; }
}

public record UpdateProductRequest
{
    [JsonPropertyName("name")]
    public string? Name { get; set; }
    [JsonPropertyName("description")]
    public string? Description { get; set; }
    [JsonPropertyName("price")]
    public Price? Price { get; set; }
    [JsonPropertyName("category")]
    public ProductCategory? Category { get; set; }
    [JsonPropertyName("isActive")]
    public bool? IsActive { get; set; }
}

public record UpdateOrderStatusRequest
{
    [JsonPropertyName("status")]
    public required OrderStatus Status { get; set; }
    [JsonPropertyName("trackingNumber")]
    public string? TrackingNumber { get; set; }
}

public record Category
{
    [JsonPropertyName("id")]
    public required Guid Id { get; set; }
    [JsonPropertyName("name")]
    public required string Name { get; set; }
    [JsonPropertyName("slug")]
    public required string Slug { get; set; }
    [JsonPropertyName("description")]
    public string? Description { get; set; }
    [JsonPropertyName("parentId")]
    public Guid? ParentId { get; set; }
    [JsonPropertyName("isActive")]
    public bool? IsActive { get; set; }
}

public record CreateCategoryRequest
{
    [JsonPropertyName("name")]
    public required string Name { get; set; }
    [JsonPropertyName("slug")]
    public required string Slug { get; set; }
    [JsonPropertyName("description")]
    public string? Description { get; set; }
    [JsonPropertyName("parentId")]
    public Guid? ParentId { get; set; }
}

public record InventoryResponse
{
    [JsonPropertyName("data")]
    public required List<Dictionary<string, object>> Data { get; set; }
}

public record UpdateInventoryRequest
{
    [JsonPropertyName("quantity")]
    public required int Quantity { get; set; }
    [JsonPropertyName("lowStockThreshold")]
    public int? LowStockThreshold { get; set; }
}

public record SalesAnalytics
{
    [JsonPropertyName("totalRevenue")]
    public required double TotalRevenue { get; set; }
    [JsonPropertyName("totalOrders")]
    public required int TotalOrders { get; set; }
    [JsonPropertyName("averageOrderValue")]
    public required double AverageOrderValue { get; set; }
    [JsonPropertyName("topProducts")]
    public List<Dictionary<string, object>> TopProducts { get; set; }
    [JsonPropertyName("period")]
    public Dictionary<string, object>? Period { get; set; }
}

public record ProductAnalytics
{
    [JsonPropertyName("totalProducts")]
    public required int TotalProducts { get; set; }
    [JsonPropertyName("activeProducts")]
    public required int ActiveProducts { get; set; }
    [JsonPropertyName("categoryBreakdown")]
    public Dictionary<string, object>? CategoryBreakdown { get; set; }
    [JsonPropertyName("lowStockProducts")]
    public List<Dictionary<string, object>> LowStockProducts { get; set; }
}

