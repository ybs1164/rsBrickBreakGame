use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_rapier2d::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
struct Health {
    hp: i32,
}

#[derive(Component)]
struct Speed {
    speed: f32,
}

#[derive(Component)]
struct Controller;

#[derive(Component)]
struct Bounced;

#[derive(Component)]
struct Brick;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Player;

#[derive(Bundle)]
struct BrickBundle {
    _b: Brick,
    health: Health,
    shape: ShapeBundle,
}

#[derive(Bundle)]
struct BallBundle {
    _b: Ball,
    speed: Speed,
    bounced: Bounced,
    shape: ShapeBundle,
}

#[derive(Bundle)]
struct PlayerBundle {
    _p: Player,
    speed: Speed,
    control: Controller,
    shape: ShapeBundle,
}

fn get_block_shape(position:Vec2, color:Color) -> ShapeBundle {
    GeometryBuilder::build_as(
        &shapes::Rectangle {
            extents: Vec2::new(100.0, 10.0),
            origin: RectangleOrigin::Center,
        },
        DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(color)),
        Transform::from_xyz(position.x, position.y, 0.0)
    )
}

fn get_circle_shape(position:Vec2, color:Color) -> ShapeBundle {
    let mut transform = Transform::from_xyz(position.x, position.y, 0.0);
    transform.rotate_z(PI);

    GeometryBuilder::build_as(
        &shapes::Circle {
            radius: 5.0,
            center: Vec2::new(0.0, 0.0),
        },
        DrawMode::Fill(bevy_prototype_lyon::prelude::FillMode::color(color)),
        transform
    )
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(ShapePlugin)
        .add_startup_system(setup_system)
        .add_system(control_system)
        .add_system(display_events)
        .run();
}

fn setup_system(mut commands: Commands) {
    let player = PlayerBundle {
        _p: Player, 
        speed: Speed { speed: 5.0 },
        control: Controller,
        shape: get_block_shape(Vec2::new(0.0, -200.0), Color::CYAN)
    };
    let ball = BallBundle {
        _b: Ball,
        speed: Speed { speed: 2.0 },
        bounced: Bounced,
        shape: get_circle_shape(Vec2::default(), Color::WHITE)
    };

    commands.spawn(Camera2dBundle::default());

    // top wall
    commands.spawn(RigidBody::Fixed)
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 250.0, 0.0)))
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Collider::cuboid(500.0, 5.0))
        .insert(Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        });

    // right wall
    commands.spawn(RigidBody::Fixed)
        .insert(TransformBundle::from(Transform::from_xyz(350.0, 0.0, 0.0)))
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Collider::cuboid(5.0, 500.0))
        .insert(Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        });

    // left wall
    commands.spawn(RigidBody::Fixed)
        .insert(TransformBundle::from(Transform::from_xyz(-350.0, 0.0, 0.0)))
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Collider::cuboid(5.0, 500.0))
        .insert(Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        });

    commands.spawn(player)
        .insert(RigidBody::KinematicPositionBased)
        .insert(ExternalImpulse {
            impulse: Vec2::new(0.0, 0.0),
            torque_impulse: 0.0,
        })
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Collider::cuboid(50.0, 5.0))
        .insert(Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Min,
        }).insert(Friction {
            coefficient: 2.0,
            combine_rule: CoefficientCombineRule::Min,
        });

    commands.spawn(ball)
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(ColliderMassProperties::Density(2.0))
        .insert(Velocity {
            linvel: Vec2::new(0.0, -200.0),
            angvel: 0.0,
        })
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Collider::ball(5.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Min,
        });

    for x in -2..3 {
        for y in 0..4 {
            let brick = BrickBundle {
                _b: Brick,
                health: Health { hp: 2 },
                shape: get_block_shape(Vec2::new((x as f32)*130.0, (y as f32)*20.0 + 100.0), Color::BISQUE)
            };

            commands.spawn(brick)
                .insert(RigidBody::KinematicPositionBased)
                .insert(Sleeping::disabled())
                .insert(Ccd::enabled())
                .insert(Collider::cuboid(50.0, 5.0))
                .insert(Restitution {
                    coefficient: 1.0,
                    combine_rule: CoefficientCombineRule::Min,
                })
                .insert(Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                });
        }
    }

}

fn control_system(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, Option<&Speed>), (With<Controller>, With<RigidBody>)>,
) {
    let (mut transform, speed) = query.single_mut();

    let control_speed = speed.map_or(1.0, |s| s.speed);

    if keyboard.pressed(KeyCode::Right) {
        transform.translation += Vec3::new(control_speed, 0.0, 0.0);
    } else if keyboard.pressed(KeyCode::Left) {
        transform.translation += Vec3::new(-control_speed, 0.0, 0.0);
    }
}

fn display_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut bricks: Query<&mut Health, With<Brick>>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(_, _, _) => {}
            CollisionEvent::Stopped(a, b, _) => {
                if let Ok(mut _health) = bricks.get_mut(*a) {
                    _health.hp -= 1;
                    if _health.hp <= 0 {
                        commands.entity(*a)
                            .despawn_recursive();
                    }
                }
                if let Ok(mut _health) = bricks.get_mut(*b) {
                    _health.hp -= 1;
                    if _health.hp <= 0 {
                        commands.entity(*b)
                            .despawn_recursive();
                    }
                }
            }
        }
    }
}