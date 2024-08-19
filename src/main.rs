use bevy::prelude::*;
use bevy_image_export::{ImageExportBundle, ImageExportPlugin, ImageExportSource};
use bevy_rapier2d::prelude::*;
use rand::Rng;

const WIDTH: u32 = 1080;
const HEIGHT: u32 = 1920;

#[derive(Component)]
struct CollisionCooldown {
    timer: Timer,
}

fn main() {
    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        .add_systems(Update, handle_collisions)
        .add_systems(Update, update_cooldowns)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera2dBundle::default());
}

fn setup_physics(mut commands: Commands) {
    let ring_group = Group::GROUP_1;
    let ball_group = Group::GROUP_2;

    let ring_radius = 250.0;
    let ring_thickness = 10.0;
    let num_segments = 36;
    let angle_step = 2.0 * std::f64::consts::PI / num_segments as f64;
    let segment_length = (ring_radius * 2.0 * std::f64::consts::PI / num_segments as f64) as f32;

    // Create the outer ring using segments
    let ring_body = commands.spawn(RigidBody::Fixed).id();

    for i in 0..num_segments {
        let angle = i as f32 * angle_step as f32; // Casting only once
        let (sin_angle, cos_angle) = angle.sin_cos(); // Compute sin and cos simultaneously
        let (x, y) = (
            ring_radius as f32 * cos_angle,
            ring_radius as f32 * sin_angle,
        );

        let segment = Collider::cuboid(ring_thickness, segment_length);
        let translation =
            Transform::from_xyz(x, y, 0.0).with_rotation(Quat::from_rotation_z(angle));

        commands.entity(ring_body).with_children(|parent| {
            parent
                .spawn(segment)
                .insert(Restitution::coefficient(1.5))
                .insert(TransformBundle {
                    local: translation,
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
        .insert(Collider::ball(10.0))
        .insert(Restitution::coefficient(1.0))
        .insert(TransformBundle::from(Transform::from_xyz(
            position.x, position.y, position.z,
        )))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(GravityScale(1.0))
        .insert(CollisionGroups::new(ball_group, ring_group)) // Ball collides with ring only
        .insert(CollisionCooldown {
            timer: Timer::new(std::time::Duration::from_secs_f32(0.2), TimerMode::Once), // 0.2-second cooldown
        });
}

fn handle_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(Entity, &mut CollisionCooldown)>,
) {
    let ring_group = Group::GROUP_1;
    let ball_group = Group::GROUP_2;

    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(collider1, collider2, _) = collision_event {
            for (entity, mut cooldown) in query.iter_mut() {
                // Check if this entity is involved in the collision
                if (*collider1 == entity || *collider2 == entity) && cooldown.timer.finished() {
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
