# Demolition Format Support

Rocket League changed the demolition data structure in **September 2024 (v2.43+)**, from `DemolishFx` to `DemolishExtended`. This library automatically detects and supports both formats.

## Format Differences

| Format | RL Version | Attribute Key | Actor ID Access |
|--------|------------|---------------|-----------------|
| `DemolishFx` | Pre-Sept 2024 | `ReplicatedDemolishGoalExplosion` | `attacker`, `victim` |
| `DemolishExtended` | Sept 2024+ | `ReplicatedDemolishExtended` | `attacker.actor`, `victim.actor` |

## How It Works

1. **Detection**: On first demolition, checks all car actors for either attribute key (tries Extended first)
2. **Caching**: Detected format is cached in `processor.demolish_format`
3. **Collection**: `get_active_demos()` uses the cached format exclusively

## API

```rust
// Format is auto-detected during processing
processor.process(&mut collector).unwrap();

// Check detected format (optional)
match processor.get_demolish_format() {
    Some(DemolishFormat::Extended) => println!("New format"),
    Some(DemolishFormat::Fx) => println!("Old format"),
    None => println!("No demolitions yet"),
}

// DemolishAttribute provides unified access
for demo in processor.get_active_demos()? {
    let attacker = demo.attacker_actor_id();
    let victim = demo.victim_actor_id();
    let attacker_vel = demo.attacker_velocity();
    let victim_vel = demo.victim_velocity();
}
```

## Adding Future Formats

1. Add constant for new attribute key in `constants.rs`
2. Add variant to `DemolishFormat` and `DemolishAttribute` enums
3. Update `detect_demolish_format()` and `get_active_demos()`
4. Implement accessor methods on `DemolishAttribute`
