# Debug Tracking Benchmarks

This document describes the benchmark tests that compare the performance of running ECS systems with and without debug tracker recording changes in components.

## Overview

The benchmarks measure the performance impact of enabling debug tracking in the Entity Component System (ECS). Debug tracking records component state changes during system execution, which is useful for debugging but may have performance implications.

## Benchmark Structure

### Test Scenarios

The benchmarks test four main scenarios:

1. **no_debug_tracking_*_entities** - Systems run without debug tracking
2. **with_debug_tracking_*_entities** - Systems run with debug tracking enabled
3. **iterator_no_debug_*_entities** - Iterator-based systems without debug tracking
4. **iterator_with_debug_*_entities** - Iterator-based systems with debug tracking

### Entity Counts

Each scenario is tested with different entity counts to measure scalability:
- 100 entities
- 500 entities  
- 1000 entities

### System Types

Three types of systems are benchmarked:

1. **velocity_system** - Modifies velocity components based on position
2. **health_system** - Modifies health components based on velocity
3. **combined_systems** - Runs both velocity and health systems sequentially

## Running Benchmarks

### Local Development

Run benchmarks locally with:

```bash
cargo bench --bench debug_tracking_benchmarks
```

This will generate detailed performance reports in `target/criterion/`.

### Continuous Integration

Benchmarks run automatically on:
- Push to main/master branch
- Pull requests
- Manual workflow dispatch

The CI workflow:
1. Runs benchmarks on Ubuntu latest
2. Generates HTML reports
3. Uploads results as artifacts
4. Comments on PRs with benchmark information

## Interpreting Results

### Performance Metrics

The benchmarks measure:
- **Execution time** - How long each system takes to run
- **Throughput** - Operations per second
- **Relative performance** - Comparison between debug/no-debug scenarios

### Expected Overhead

Debug tracking introduces overhead due to:
- Component state snapshots before system execution
- Diff calculation after system execution 
- Memory allocation for tracking data
- Serialization of component changes

### Performance Guidelines

- **Development**: Debug tracking can be enabled for debugging without significant impact on small entity counts
- **Production**: Disable debug tracking for optimal performance
- **Testing**: Use benchmarks to validate performance characteristics

## Implementation Details

### Debug Tracking Features

When enabled, debug tracking:
1. Takes snapshots of mutable components before system execution
2. Compares component state after system execution
3. Records differences using the `Diffable` trait
4. Stores change history with frame numbers and system names

### Benchmark Configuration

- Uses `criterion` crate for statistical analysis
- Runs multiple iterations for accuracy
- Uses `black_box` to prevent compiler optimizations
- Configures appropriate batch sizes for reliable measurements

## Files

- `benches/debug_tracking_benchmarks.rs` - Benchmark implementation
- `.github/workflows/benchmarks.yml` - CI workflow
- This documentation file

## Future Improvements

Potential enhancements:
- Memory usage benchmarks
- Different component types and sizes
- Parallel system execution benchmarks
- Profiling integration for detailed analysis