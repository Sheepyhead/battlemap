use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_inspector_egui::WorldInspectorPlugin;
use heron::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
        .add_plugin(WorldInspectorPlugin::default())
        .add_startup_system(setup)
        .add_system(camera_look)
        .add_system(player_move)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle((
            Transform::default(),
            GlobalTransform::default(),
            CollisionShape::Cylinder {
                half_height: 1.0,
                radius: 0.5,
            },
            RigidBody::Dynamic,
            RotationConstraints::lock(),
            Name::new("Player"),
        ))
        .insert_bundle((Player,))
        .with_children(|parent| {
            let mut camera = PerspectiveCameraBundle::default();
            camera.transform.translation.y += 0.5;
            parent.spawn_bundle(camera);
        });

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(shape::Plane { size: 100.0 }.into()),
            material: materials.add(Color::GREEN.into()),
            ..PbrBundle::default()
        })
        .insert_bundle((
            CollisionShape::Cuboid {
                half_extends: Vec3::new(50.0, 0.5, 50.0),
                border_radius: None,
            },
            RigidBody::Static,
            Name::new("Ground"),
        ));

    commands.spawn_bundle((
        CollisionShape::Cuboid {
            half_extends: Vec3::splat(0.5),
            border_radius: None,
        },
        Transform::from_xyz(0.0, 10.0, -5.0),
        GlobalTransform::default(),
        RigidBody::Dynamic,
    ));
}

#[derive(Component)]
struct Player;

fn camera_look(
    mut events: EventReader<MouseMotion>,
    mut state: Local<Vec2>,
    player: Query<&Children, With<Player>>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<Parent>)>,
) {
    for MouseMotion { delta } in events.iter() {
        for children in player.iter() {
            for child in children.iter() {
                if let Ok(mut camera) = cameras.get_mut(*child) {
                    state.x -= delta.y.to_radians();
                    state.y -= delta.x.to_radians();

                    state.x = state.x.clamp(-1.54, 1.54);

                    // Order is important to prevent unintended roll
                    camera.rotation = Quat::from_axis_angle(Vec3::Y, state.y)
                        * Quat::from_axis_angle(Vec3::X, state.x);
                }
            }
        }
    }
}

fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player: Query<(&mut Transform, &Children), With<Player>>,
    camera: Query<&GlobalTransform, With<Parent>>,
) {
    let (mut player, children) = player.single_mut();
    for child in children.iter() {
        if let Ok(camera) = camera.get(*child) {
            let mut velocity = Vec3::ZERO;
            let local_z = camera.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match key {
                    KeyCode::Comma => velocity += forward,
                    KeyCode::O => velocity -= forward,
                    KeyCode::A => velocity -= right,
                    KeyCode::E => velocity += right,
                    _ => (),
                }
            }

            velocity = velocity.normalize_or_zero();

            player.translation += velocity * time.delta_seconds();
        }
    }
}
