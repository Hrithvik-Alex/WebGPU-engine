use std::time::Duration;

use cgmath::{Vector2, Zero};
use log::debug;

use crate::{
    component::{self, Component, EntityMap, PositionComponent},
    input::InputHandler,
    state, utils,
};

pub struct BoundingBox {
    pub bottom_left: Vector2<f32>,
    pub top_right: Vector2<f32>,
}

pub struct ColliderBoxComponent {
    pub bounding_box: BoundingBox,
}

impl Component for ColliderBoxComponent {
    fn name(&self) -> String {
        return "ColliderBox".to_string();
    }
}

#[derive(PartialEq)]
pub struct PhysicsComponent {
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
    pub last_grounded_time: Option<Duration>,
}

impl Component for PhysicsComponent {
    fn name(&self) -> String {
        return "Physics".to_string();
    }
}

impl PhysicsComponent {
    pub fn new() -> Self {
        Self {
            velocity: cgmath::Vector2::new(0.0, 0.0),
            acceleration: cgmath::Vector2::new(0.0, 0.0),
            last_grounded_time: None,
        }
    }
}

pub struct PhysicsSystem {
    tick_duration: Duration,
}

impl PhysicsSystem {
    const MOVEMENT_SPEED: f32 = 100.;

    const JUMP_VELOCITY: f32 = 300.;
    const JUMP_ACCELERATION: f32 = 600.;

    const COYOTE_TIME: Duration = Duration::from_millis(300);

    pub fn new(tick_duration: Duration) -> Self {
        Self { tick_duration }
    }

    fn is_colliding(a: &BoundingBox, b: &BoundingBox) -> bool {
        !(a.top_right.x <= b.bottom_left.x
            || a.bottom_left.x >= b.top_right.x
            || a.top_right.y <= b.bottom_left.y
            || a.bottom_left.y >= b.top_right.y)
    }

    fn is_touching(a: &BoundingBox, b: &BoundingBox) -> bool {
        !(a.top_right.x < b.bottom_left.x
            || a.bottom_left.x > b.top_right.x
            || a.top_right.y < b.bottom_left.y
            || a.bottom_left.y > b.top_right.y)
    }

    fn get_collision_delta(a: &BoundingBox, b: &BoundingBox) -> (Vector2<f32>, f32) {
        let horizontal_depth = f32::min(
            a.top_right.x - b.bottom_left.x,
            b.top_right.x - a.bottom_left.x,
        );
        let vertical_depth = f32::min(
            a.top_right.y - b.bottom_left.y,
            b.top_right.y - a.bottom_left.y,
        );

        if horizontal_depth < vertical_depth {
            if a.bottom_left.x < b.bottom_left.x {
                return (Vector2::unit_x(), horizontal_depth);
            } else {
                return (Vector2::unit_x() * -1.0, horizontal_depth);
            }
        } else {
            if a.bottom_left.y < b.bottom_left.y {
                return (Vector2::unit_y(), vertical_depth);
            } else {
                return (Vector2::unit_y() * -1.0, vertical_depth);
            }
        }
        // return Vector2::new(0.0, 0.0);
    }

    pub fn update(
        &self,
        input_handler: &InputHandler,
        position_components: &mut EntityMap<PositionComponent>,
        collider_box_components: &mut EntityMap<ColliderBoxComponent>,
        metadata_components: &mut EntityMap<component::MetadataComponent>,
        physics_components: &mut EntityMap<PhysicsComponent>,
        collectible_components: &mut EntityMap<component::CollectibleComponent>,
        current_time: Duration,
    ) {
        let tick_secs = self.tick_duration.as_secs_f32();
        // let position_delta = cgmath::Vector2::new(x, y) * Self::MOVEMENT_SPEED * tick_secs;

        let collider_deltas = utils::zip4_entities_1immut(
            position_components,
            physics_components,
            metadata_components,
            collider_box_components,
        )
        .map(
            |(
                e1,
                position_component,
                physics_component,
                metadata_component,
                collider_box_component1,
            )| {
                let Some(position_component) = position_component else {
                    return cgmath::Vector2::zero();
                };
                let Some(physics_component) = physics_component else {
                    return cgmath::Vector2::zero();
                };

                let metadata_component = metadata_component.as_mut().unwrap();

                if metadata_component.is_controllable() {
                    if input_handler.up_pressed {
                        if metadata_component.can_jump() {
                            debug!("JUMPY BOI");
                            metadata_component.set_jump(false);
                            physics_component.velocity.y = Self::JUMP_VELOCITY;
                            physics_component.acceleration.y = -1. * Self::JUMP_ACCELERATION;
                        }
                    }
                    // if input_handler.down_pressed {
                    //     update_position(0., -1.)
                    // }
                    if input_handler.left_pressed {
                        physics_component.velocity.x = -1. * Self::MOVEMENT_SPEED;
                        physics_component.acceleration.x = -1. * Self::MOVEMENT_SPEED;
                    } else if input_handler.right_pressed {
                        physics_component.velocity.x = Self::MOVEMENT_SPEED;
                        physics_component.acceleration.x = Self::MOVEMENT_SPEED;
                    } else {
                        physics_component.velocity.x = 0.;
                        physics_component.acceleration.x = 0.;
                    }
                };

                physics_component.velocity += physics_component.acceleration * tick_secs;
                let delta = physics_component.velocity * tick_secs;
                let mut delta_add = delta;

                if (*physics_component == PhysicsComponent::new()) {
                    return cgmath::Vector2::zero();
                }

                let Some(collider_box_component1) = collider_box_component1 else {
                    return cgmath::Vector2::zero();
                };
                let new_collision_box = ColliderBoxComponent {
                    bounding_box: BoundingBox {
                        bottom_left: collider_box_component1.bounding_box.bottom_left + delta,
                        top_right: collider_box_component1.bounding_box.top_right + delta,
                    },
                };

                let mut is_grounded = false;
                // TODO: implement better collision detection, this is O(N^2) lol
                let collision_detected = collider_box_components
                    .iter()
                    .zip(collectible_components.iter_mut())
                    .fold(
                        Vector2::zero(),
                        |mut collision_dir, ((e2, box2), (_, collectible))| {
                            if (e1 != e2) {
                                box2.as_ref().map(|box2| {
                                    let (direction, scale) = Self::get_collision_delta(
                                        &new_collision_box.bounding_box,
                                        &box2.bounding_box,
                                    );
                                    if (Self::is_colliding(
                                        &new_collision_box.bounding_box,
                                        &box2.bounding_box,
                                    )) {
                                        collision_dir += direction * scale;
                                    }

                                    if (Self::is_touching(
                                        &new_collision_box.bounding_box,
                                        &box2.bounding_box,
                                    )) {
                                        if direction == (Vector2::unit_y() * -1.) {
                                            is_grounded = true;
                                        }
                                    }
                                });

                                collectible.as_mut().map(|collectible| {
                                    if Self::is_colliding(
                                        &new_collision_box.bounding_box,
                                        &collectible.bounding_box,
                                    ) {
                                        collectible.is_collected = true;
                                    }
                                });
                            }
                            collision_dir
                        },
                    );

                if (is_grounded) {
                    if physics_component.last_grounded_time.is_some() {
                        physics_component.last_grounded_time = None;
                    }

                    metadata_component.set_jump(true);
                    physics_component.acceleration.y = 0.;
                    physics_component.velocity.y = 0.;
                } else {
                    if (physics_component.last_grounded_time.is_none()) {
                        physics_component.last_grounded_time = Some(current_time);
                    }

                    physics_component.acceleration.y = -1. * Self::JUMP_ACCELERATION;

                    assert!(physics_component.last_grounded_time.is_some());
                    let grounded_start_time = physics_component.last_grounded_time.unwrap();

                    if current_time - grounded_start_time > Self::COYOTE_TIME {
                        metadata_component.set_jump(false);
                    }
                }

                if (physics_component.last_grounded_time.is_none() && !is_grounded) {
                    physics_component.last_grounded_time = Some(current_time);
                } else if physics_component.last_grounded_time.is_some() && is_grounded {
                    physics_component.last_grounded_time = None;
                }

                if !collision_detected.is_zero() {
                    if collision_detected.y != 0. {
                        delta_add.y -= collision_detected.y;

                        // if collision_detected.y < 0. {
                        //     metadata_component.set_jump(true);
                        //     physics_component.acceleration.y = 0.;
                        //     physics_component.velocity.y = 0.;
                        // }
                    }

                    if collision_detected.x != 0. {
                        delta_add.x -= collision_detected.x;
                    }
                }
                position_component.position += delta_add;
                delta_add
            },
        )
        .collect::<Vec<Vector2<f32>>>();

        // rust limitation of mutable references so I need to do this :(( maybe I can use itertools?
        collider_deltas
            .iter()
            .zip(collider_box_components.iter_mut())
            .for_each(|(delta, (_, collider_box_component))| {
                if let Some(collider_box_component) = collider_box_component {
                    collider_box_component.bounding_box.bottom_left += *delta;
                    collider_box_component.bounding_box.top_right += *delta;
                }
            });
    }
}
