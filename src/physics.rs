use std::time::Duration;

use cgmath::Vector2;

use crate::{
    component::{Component, EntityMap, PositionComponent},
    input::InputHandler,
};

pub struct ColliderBoxComponent {
    pub bottom_left: Vector2<f32>,
    pub top_right: Vector2<f32>,
}

impl Component for ColliderBoxComponent {
    fn name(&self) -> String {
        return "ColliderBox".to_string();
    }
}

impl ColliderBoxComponent {}

pub struct PhysicsSystem {
    tick_duration: Duration,
}

impl PhysicsSystem {
    const MOVEMENT_SPEED: f32 = 50.;

    pub fn new(tick_duration: Duration) -> Self {
        Self { tick_duration }
    }

    fn is_colliding(a: &ColliderBoxComponent, b: &ColliderBoxComponent) -> bool {
        !(a.top_right.x <= b.bottom_left.x
            || a.bottom_left.x >= b.top_right.x
            || a.top_right.y <= b.bottom_left.y
            || a.bottom_left.y >= b.top_right.y)
    }

    pub fn update(
        &self,
        input_handler: &InputHandler,
        position_components: &mut EntityMap<PositionComponent>,
        collider_box_components: &mut EntityMap<ColliderBoxComponent>,
    ) {
        let tick_secs = self.tick_duration.as_secs_f32();
        let mut update_position = |x: f32, y: f32| {
            let delta = cgmath::Vector2::new(x, y) * Self::MOVEMENT_SPEED * tick_secs;
            let collider_deltas = position_components
                .iter_mut()
                .zip(collider_box_components.iter())
                .map(|((_, position_component), (e1, collider_box_component1))| {
                    let Some(position_component) = position_component else {
                        return cgmath::Vector2::new(0., 0.);
                    };
                    if !position_component.is_controllable {
                        return cgmath::Vector2::new(0., 0.);
                    };

                    let mut delta_add = delta;

                    let Some(collider_box_component1) = collider_box_component1 else {
                        return cgmath::Vector2::new(0., 0.);
                    };
                    let new_collision_box = ColliderBoxComponent {
                        bottom_left: collider_box_component1.bottom_left + delta,
                        top_right: collider_box_component1.top_right + delta,
                    };

                    // TODO: implement better collision detection, this is O(N^2) lol

                    let collision_detected = collider_box_components.iter().any(|(e2, box2)| {
                        box2.as_ref().map_or(false, |box2| {
                            e1 != e2 && Self::is_colliding(&new_collision_box, &box2)
                        })
                    });

                    if collision_detected {
                        delta_add -= delta;
                    }
                    position_component.position += delta_add;
                    delta_add
                })
                .collect::<Vec<Vector2<f32>>>();

            // rust limitation of mutable references so I need to do this :(( maybe I can use itertools?
            collider_deltas
                .iter()
                .zip(collider_box_components.iter_mut())
                .for_each(|(delta, (_, collider_box_component))| {
                    if let Some(collider_box_component) = collider_box_component {
                        collider_box_component.bottom_left += *delta;
                        collider_box_component.top_right += *delta;
                    }
                });
        };
        if input_handler.up_pressed {
            update_position(0., 1.)
        }
        if input_handler.down_pressed {
            update_position(0., -1.)
        }
        if input_handler.left_pressed {
            update_position(-1., 0.)
        }
        if input_handler.right_pressed {
            update_position(1., 0.)
        }
    }
}
