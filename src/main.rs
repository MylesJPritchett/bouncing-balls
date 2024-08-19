use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

struct CollisionCooldown {
    timer: Timer,
}

impl Component for CollisionCooldown {} // Derive Component trait

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_startup_systems(setup_graphics)
        .add_startup_systems(setup_physics)
        .add_systems(handle_collisions)
        .add_systems(update_cooldowns)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera2dBundle::default());
}

fn setup_physics(mut commands: Commands) {
    let ring_radius = 200.0;
    let ring_thickness = 10.0;
    let num_segments = 36;
    let angle_step = 2.0 * std::f64::consts::PI / num_segments as f64;

    let ring_group = Group::GROUP_1;
    let ball_group = Group::GROUP_2;

    /* Create the outer ring using segments */
    let ring_body = commands.spawn(RigidBody::Fixed).id();

    for i in 0..num_segments {
        let angle = i as f64 * angle_step;
        let x = ring_radius * angle.cos();
        let y = ring_radius * angle.sin();

        let segment = Collider::cuboid(ring_thickness, 5.0);
        let translation = Transform::from_xyz(x as f32, y as f32, 0.0);
        let rotation = Quat::from_rotation_z(angle as f32);

        commands.entity(ring_body).with_children(|parent| {
            parent
                .spawn(segment)
                .insert(Restitution::coefficient(1.0))
                .insert(TransformBundle {
                    local: translation.with_rotation(rotation),
                    ..Default::default()
                })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(CollisionGroups::new(ring_group, ball_group)); // Ring collides with balls only
        });
    }

    /* Create the initial bouncing ball with restitution */
    spawn_ball(
        &mut commands,
        Vec3::new(0.0, 100.0, 0.0),
        ring_group,
        ball_group,
    );
}

fn spawn_ball(commands: &mut Commands, position: Vec3, ring_group: Group, ball_group: Group) {
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(50.0))
        .insert(Restitution::coefficient(1.0))
        .insert(TransformBundle::from(Transform::from_xyz(
            position.x, position.y, position.z,
        )))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(CollisionGroups::new(ball_group, ring_group)) // Ball collides with ring only
        .insert(CollisionCooldown {
            timer: Timer::from_seconds(0.2, TimerMode::Once), // 0.2-second cooldown
        });
}

fn handle_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(Entity, &mut CollisionCooldown)>,
) {
    let ring_group = Group::GROUP_1;
    let ball_group = Group::GROUP_2;

    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(collider1, collider2, _) = collision_event {
            for (entity, mut cooldown) in query.iter_mut() {
                if cooldown.timer.finished() {
                    // Reset the cooldown
                    cooldown.timer.reset();

                    // Spawn a new ball at a random position
                    let x: f32 = rand::thread_rng().gen_range(-100.0..100.0);
                    let y: f32 = rand::thread_rng().gen_range(0.0..100.0);
                    spawn_ball(&mut commands, Vec3::new(x, y, 0.0), ring_group, ball_group);

                    // Break to prevent multiple spawns from the same collision event
                    break;
                }
            }
        }
    }
}

fn update_cooldowns(time: Res<Time>, mut query: Query<&mut CollisionCooldown>) {
    for mut cooldown in query.iter_mut() {
        cooldown.timer.tick(time.delta());
    }
}
