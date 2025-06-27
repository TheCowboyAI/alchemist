# Location Domain

## Overview

The Location Domain manages geographic and spatial information within CIM, providing location-based services, geospatial analysis, and location-aware features. It handles everything from simple addresses to complex geographic regions and spatial relationships.

## Key Concepts

### Location
- **Definition**: A point or area in geographic space
- **Types**: Address, coordinates, region, landmark, zone
- **Properties**: Name, type, geometry, metadata
- **Precision**: From country-level to exact coordinates

### Address
- **Definition**: Structured postal address information
- **Components**: Street, city, state, postal code, country
- **Validation**: Format checking, geocoding, standardization
- **Localization**: Country-specific address formats

### Geographic Coordinates
- **Definition**: Precise latitude/longitude positions
- **Systems**: WGS84, local coordinate systems
- **Properties**: Latitude, longitude, altitude, accuracy
- **Operations**: Distance calculation, bounding box queries

### Region
- **Definition**: Bounded geographic area
- **Types**: Administrative, custom, geometric
- **Properties**: Boundary polygon, area, population
- **Hierarchy**: Country → State → City → District

## Domain Events

### Commands
- `cmd.location.create_location` - Define new location
- `cmd.location.geocode_address` - Convert address to coordinates
- `cmd.location.reverse_geocode` - Convert coordinates to address
- `cmd.location.create_region` - Define geographic region
- `cmd.location.update_location` - Modify location details

### Events
- `event.location.location_created` - New location defined
- `event.location.address_geocoded` - Address converted
- `event.location.region_created` - New region defined
- `event.location.location_updated` - Location modified
- `event.location.location_entered` - Entity entered location

### Queries
- `query.location.find_nearby` - Search by proximity
- `query.location.get_in_region` - Find locations in area
- `query.location.calculate_distance` - Distance between points
- `query.location.get_address` - Retrieve formatted address

## API Reference

### LocationAggregate
```rust
pub struct LocationAggregate {
    pub id: LocationId,
    pub name: String,
    pub location_type: LocationType,
    pub geometry: Geometry,
    pub address: Option<Address>,
    pub metadata: LocationMetadata,
}
```

### Key Methods
- `create_location()` - Initialize location
- `geocode_address()` - Address to coordinates
- `reverse_geocode()` - Coordinates to address
- `find_nearby()` - Proximity search
- `contains_point()` - Point-in-region test

## Location Types

### Address Management
```rust
// Create structured address
let address = CreateAddress {
    street_lines: vec![
        "123 Main Street".to_string(),
        "Suite 456".to_string(),
    ],
    city: "San Francisco".to_string(),
    state_province: "CA".to_string(),
    postal_code: "94105".to_string(),
    country: "US".to_string(),
};

// Geocode to coordinates
let geocoded = GeocodeAddress {
    address,
    precision: GeocodePrecision::Rooftop,
};

// Result includes coordinates and confidence
let result = GeocodedLocation {
    coordinates: Coordinates {
        latitude: 37.7749,
        longitude: -122.4194,
        altitude: Some(15.0),
    },
    accuracy_meters: 10.0,
    confidence: 0.95,
};
```

### Coordinate Operations
```rust
// Create coordinate-based location
let coordinates = Coordinates {
    latitude: 40.7128,
    longitude: -74.0060,
    altitude: Some(10.0),
};

// Calculate distance between points
let distance = calculate_distance(
    &coordinates1,
    &coordinates2,
    DistanceUnit::Kilometers,
);

// Find locations within radius
let nearby = FindNearby {
    center: coordinates,
    radius_km: 5.0,
    location_types: vec![LocationType::Restaurant, LocationType::Cafe],
    max_results: 20,
};
```

### Region Definition
```rust
// Define geographic region
let region = CreateRegion {
    name: "Downtown District".to_string(),
    region_type: RegionType::Administrative,
    boundary: Polygon {
        exterior: vec![
            Coordinates { lat: 37.7749, lng: -122.4194 },
            Coordinates { lat: 37.7751, lng: -122.4180 },
            Coordinates { lat: 37.7740, lng: -122.4175 },
            Coordinates { lat: 37.7738, lng: -122.4189 },
        ],
        holes: vec![], // Interior exclusions
    },
    metadata: RegionMetadata {
        population: Some(25000),
        area_sq_km: 2.5,
        timezone: "America/Los_Angeles".to_string(),
    },
};

// Test point containment
let contains = region.contains_point(&test_coordinates);

// Find all locations in region
let in_region = GetLocationsInRegion {
    region_id,
    location_types: None, // All types
};
```

## Spatial Indexing

### R-Tree Index
```rust
// Spatial index for efficient queries
pub struct SpatialIndex {
    rtree: RTree<LocationId, Coordinates>,
}

impl SpatialIndex {
    pub fn find_within_bounds(
        &self,
        bounds: BoundingBox,
    ) -> Vec<LocationId> {
        self.rtree
            .locate_in_envelope(&bounds.to_envelope())
            .map(|item| item.data)
            .collect()
    }

    pub fn k_nearest_neighbors(
        &self,
        point: &Coordinates,
        k: usize,
    ) -> Vec<(LocationId, f64)> {
        self.rtree
            .nearest_neighbor_iter(point)
            .take(k)
            .map(|item| (item.data, item.distance()))
            .collect()
    }
}
```

### Geohashing
```rust
// Geohash for proximity grouping
let geohash = coordinates.to_geohash(precision: 8);
// "9q8yyz8g" - 8 character precision (~19m)

// Find locations with similar geohash prefix
let nearby_hashes = geohash.neighbors();
let nearby_locations = query_by_geohash_prefix(&geohash[..6]);
```

## Location Services

### Geocoding Service
```rust
pub trait GeocodingService {
    async fn geocode(
        &self,
        address: &Address,
    ) -> Result<GeocodedLocation, GeocodingError>;

    async fn reverse_geocode(
        &self,
        coordinates: &Coordinates,
    ) -> Result<Address, GeocodingError>;

    async fn batch_geocode(
        &self,
        addresses: Vec<Address>,
    ) -> Result<Vec<GeocodedLocation>, GeocodingError>;
}
```

### Route Planning
```rust
// Calculate route between locations
let route = PlanRoute {
    start: location_a,
    end: location_b,
    waypoints: vec![location_c, location_d],
    mode: TravelMode::Driving,
    avoid: vec![AvoidType::Tolls, AvoidType::Highways],
};

let route_result = RouteResult {
    distance_km: 45.2,
    duration_minutes: 38,
    steps: vec![
        RouteStep {
            instruction: "Head north on Main St".to_string(),
            distance_m: 500,
            duration_s: 120,
        },
        // ... more steps
    ],
    polyline: encoded_polyline,
};
```

### Geofencing
```rust
// Create geofence
let geofence = CreateGeofence {
    name: "Office Perimeter".to_string(),
    boundary: region_boundary,
    trigger_on: vec![GeofenceTrigger::Enter, GeofenceTrigger::Exit],
    active_hours: Some(BusinessHours {
        start: "08:00".to_string(),
        end: "18:00".to_string(),
        days: vec![Weekday::Mon, Weekday::Tue, /* ... */],
    }),
};

// Monitor geofence events
on_event("location.geofence.entered", |event| {
    notify_security(event.entity_id, event.geofence_id);
});
```

## Integration Patterns

### Person Location Tracking
```rust
// Track person's current location
let update = UpdatePersonLocation {
    person_id,
    location: CurrentLocation {
        coordinates,
        accuracy: 5.0,
        timestamp: SystemTime::now(),
        source: LocationSource::GPS,
    },
};

// Location history
let history = GetLocationHistory {
    person_id,
    start_time,
    end_time,
    max_points: 1000,
};
```

### Asset Management
```rust
// Track asset locations
let asset_location = AssetLocation {
    asset_id,
    location_id,
    status: LocationStatus::InTransit,
    last_seen: SystemTime::now(),
    next_destination: Some(destination_id),
};

// Find assets in area
let assets = FindAssetsInRegion {
    region_id,
    asset_types: vec![AssetType::Vehicle, AssetType::Equipment],
    status_filter: Some(LocationStatus::Active),
};
```

## Visualization

### Map Integration
```rust
// Convert to map markers
impl From<Location> for MapMarker {
    fn from(location: Location) -> Self {
        MapMarker {
            position: location.coordinates,
            title: location.name,
            icon: location.location_type.icon(),
            info_window: location.format_info(),
        }
    }
}

// Heatmap data
let heatmap = GenerateHeatmap {
    locations: location_ids,
    weight_by: HeatmapWeight::Frequency,
    radius: 1000.0, // meters
    gradient: HeatmapGradient::default(),
};
```

## Use Cases

### Delivery and Logistics
- Route optimization
- Delivery tracking
- Service area definition
- Driver location monitoring

### Retail and Services
- Store locator
- Service area coverage
- Customer proximity analysis
- Location-based promotions

### Emergency Services
- Incident location tracking
- Resource positioning
- Response time analysis
- Coverage area planning

### IoT and Sensors
- Device location management
- Sensor data correlation
- Environmental monitoring
- Movement pattern analysis

## Performance Characteristics

- **Location Capacity**: 100M+ locations
- **Proximity Search**: <10ms with spatial index
- **Geocoding**: <200ms per address
- **Region Queries**: <50ms for point-in-polygon

## Best Practices

1. **Coordinate Precision**: Store appropriate precision for use case
2. **Spatial Indexing**: Use R-tree or geohash for large datasets
3. **Caching**: Cache geocoding results to reduce API calls
4. **Privacy**: Implement location privacy controls
5. **Accuracy**: Always include accuracy/confidence metrics

## Related Domains

- **Person Domain**: Personal location tracking
- **Organization Domain**: Facility locations
- **Workflow Domain**: Location-based workflows
- **Policy Domain**: Location-based access control 