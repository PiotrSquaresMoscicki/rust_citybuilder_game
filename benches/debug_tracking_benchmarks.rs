use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_citybuilder_game::ecs::{World, Mut, EntityIterator};
use rust_citybuilder_game::examples::{Position, Velocity, Health};
use std::any::TypeId;

/// Create a world with many entities for benchmarking
fn create_benchmark_world(entity_count: usize) -> World {
    let mut world = World::new();
    
    for i in 0..entity_count {
        let entity = world.create_entity();
        world.add_component(entity, Position::new(i as f32, (i * 2) as f32));
        world.add_component(entity, Velocity::new((i % 3) as f32, (i % 5) as f32));
        world.add_component(entity, Health::new(100));
    }
    
    world
}

/// Benchmark system that modifies velocity components
fn velocity_modification_system(world: &World) {
    let ent_it = world.iter_entities::<Position, Mut<Velocity>>();
    for (position, mut velocity) in ent_it {
        // Apply some computation based on position
        velocity.dx = velocity.dx * 0.99 + position.x * 0.01;
        velocity.dy = velocity.dy * 0.99 + position.y * 0.01;
    }
}

/// Benchmark system that modifies health components  
fn health_modification_system(world: &World) {
    let ent_it = world.iter_entities::<Velocity, Mut<Health>>();
    for (velocity, mut health) in ent_it {
        // Apply damage based on velocity magnitude
        let speed = (velocity.dx * velocity.dx + velocity.dy * velocity.dy).sqrt();
        if speed > 5.0 {
            health.damage(1);
        }
    }
}

/// Combined system that runs both velocity and health systems
fn combined_systems(world: &World) {
    velocity_modification_system(world);
    health_modification_system(world);
}

/// Closure version of velocity system for ECS registration
fn velocity_modification_system_closure(ent_it: EntityIterator<Position, Mut<Velocity>>) {
    for (position, mut velocity) in ent_it {
        velocity.dx = velocity.dx * 0.99 + position.x * 0.01;
        velocity.dy = velocity.dy * 0.99 + position.y * 0.01;
    }
}

/// Closure version of health system for ECS registration  
fn health_modification_system_closure(ent_it: EntityIterator<Velocity, Mut<Health>>) {
    for (velocity, mut health) in ent_it {
        let speed = (velocity.dx * velocity.dx + velocity.dy * velocity.dy).sqrt();
        if speed > 5.0 {
            health.damage(1);
        }
    }
}

fn bench_systems_without_debug_tracking(c: &mut Criterion) {
    let entity_counts = [100, 500, 1000];
    
    for &count in &entity_counts {
        let mut group = c.benchmark_group(format!("no_debug_tracking_{}_entities", count));
        
        group.bench_function("velocity_system", |b| {
            let world = create_benchmark_world(count);
            b.iter(|| {
                black_box(velocity_modification_system(&world));
            });
        });
        
        group.bench_function("health_system", |b| {
            let world = create_benchmark_world(count);
            b.iter(|| {
                black_box(health_modification_system(&world));
            });
        });
        
        group.bench_function("combined_systems", |b| {
            let world = create_benchmark_world(count);
            b.iter(|| {
                black_box(combined_systems(&world));
            });
        });
        
        group.finish();
    }
}

fn bench_systems_with_debug_tracking(c: &mut Criterion) {
    let entity_counts = [100, 500, 1000];
    
    for &count in &entity_counts {
        let mut group = c.benchmark_group(format!("with_debug_tracking_{}_entities", count));
        
        group.bench_function("velocity_system", |b| {
            b.iter_batched(
                || {
                    let mut world = create_benchmark_world(count);
                    world.enable_debug_tracking();
                    world.next_frame();
                    world
                },
                |mut world| {
                    black_box(world.run_system_with_debug(
                        "velocity_system",
                        velocity_modification_system,
                        &[TypeId::of::<Velocity>()]
                    ));
                },
                criterion::BatchSize::SmallInput
            );
        });
        
        group.bench_function("health_system", |b| {
            b.iter_batched(
                || {
                    let mut world = create_benchmark_world(count);
                    world.enable_debug_tracking();
                    world.next_frame();
                    world
                },
                |mut world| {
                    black_box(world.run_system_with_debug(
                        "health_system", 
                        health_modification_system,
                        &[TypeId::of::<Health>()]
                    ));
                },
                criterion::BatchSize::SmallInput
            );
        });
        
        group.bench_function("combined_systems", |b| {
            b.iter_batched(
                || {
                    let mut world = create_benchmark_world(count);
                    world.enable_debug_tracking();
                    world.next_frame();
                    world
                },
                |mut world| {
                    black_box(world.run_system_with_debug(
                        "velocity_system",
                        velocity_modification_system,
                        &[TypeId::of::<Velocity>()]
                    ));
                    black_box(world.run_system_with_debug(
                        "health_system",
                        health_modification_system, 
                        &[TypeId::of::<Health>()]
                    ));
                },
                criterion::BatchSize::SmallInput
            );
        });
        
        group.finish();
    }
}

fn bench_iterator_systems_without_debug_tracking(c: &mut Criterion) {
    let entity_counts = [100, 500, 1000];
    
    for &count in &entity_counts {
        let mut group = c.benchmark_group(format!("iterator_no_debug_{}_entities", count));
        
        group.bench_function("run_systems", |b| {
            b.iter_batched(
                || {
                    let mut world = create_benchmark_world(count);
                    
                    // Add velocity system
                    world.add_single_iterator_system(
                        velocity_modification_system_closure,
                        "velocity_system"
                    );
                    
                    // Add health system
                    world.add_single_iterator_system(
                        health_modification_system_closure,
                        "health_system"
                    );
                    
                    world
                },
                |world| {
                    black_box(world.run_systems());
                },
                criterion::BatchSize::SmallInput
            );
        });
        
        group.finish();
    }
}

fn bench_iterator_systems_with_debug_tracking(c: &mut Criterion) {
    let entity_counts = [100, 500, 1000];
    
    for &count in &entity_counts {
        let mut group = c.benchmark_group(format!("iterator_with_debug_{}_entities", count));
        
        group.bench_function("run_systems", |b| {
            b.iter_batched(
                || {
                    let mut world = create_benchmark_world(count);
                    world.enable_debug_tracking();
                    world.next_frame();
                    
                    // Add velocity system
                    world.add_single_iterator_system(
                        velocity_modification_system_closure,
                        "velocity_system"
                    );
                    
                    // Add health system
                    world.add_single_iterator_system(
                        health_modification_system_closure,
                        "health_system"
                    );
                    
                    world
                },
                |mut world| {
                    black_box(world.run_iterator_systems_with_debug());
                },
                criterion::BatchSize::SmallInput
            );
        });
        
        group.finish();
    }
}

criterion_group!(
    benches,
    bench_systems_without_debug_tracking,
    bench_systems_with_debug_tracking,
    bench_iterator_systems_without_debug_tracking,
    bench_iterator_systems_with_debug_tracking
);
criterion_main!(benches);