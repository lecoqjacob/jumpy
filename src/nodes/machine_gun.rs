use macroquad::{
    experimental::{
        animation::{
            AnimatedSprite,
            Animation,
        },
        scene::{
            RefMut,
            Node,
            Handle,
            HandleUntyped,
        },
        coroutines::{
            Coroutine,
            start_coroutine,
            wait_seconds,
        },
        collections::storage,
    },
    color,
    audio::play_sound_once,
    prelude::*
};

use crate::{
    Resources,
    nodes::{
        sproinger::Sproingable,
        player::{
            Player,
            capabilities,
            PhysicsBody,
            Weapon,
        }
    }
};

pub struct MachineGun {
    pub sprite: AnimatedSprite,
    pub fx_sprite: AnimatedSprite,
    pub fx_active: bool,

    pub body: PhysicsBody,

    pub thrown: bool,

    pub bullets: i32,

    origin_pos: Vec2,
    pub deadly_dangerous: bool,
}

impl MachineGun {
    pub const GUN_THROWBACK: f32 = 75.0;
    pub const FIRE_INTERVAL: f32 = 0.0025; // Time in animation lock between bullets
    pub const MAX_BULLETS: i32 = 20;

    pub fn new(facing: bool, pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            92,
            32,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "shoot".to_string(),
                    row: 1,
                    frames: 3,
                    fps: 25,
                },
            ],
            false,
        );

        let fx_sprite = AnimatedSprite::new(
            92,
            32,
            &[Animation {
                name: "shoot".to_string(),
                row: 2,
                frames: 2,
                fps: 15,
            }],
            false,
        );

        let body = PhysicsBody {
            pos,
            facing,
            angle: 0.0,
            speed: vec2(0., 0.),
            collider: None,
            on_ground: false,
            last_frame_on_ground: false,
            have_gravity: true,
            bouncyness: 0.0,
        };

        MachineGun {
            sprite,
            fx_sprite,
            fx_active: false,
            body,
            thrown: false,
            bullets: Self::MAX_BULLETS,
            origin_pos: pos,
            deadly_dangerous: false,
        }
    }

    fn draw_hud(&self) {
        let line_height = 16.0;
        let line_spacing = 1.0;
        let line_thickness = 2.0;
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..Self::MAX_BULLETS {
            let x = self.body.pos.x - 15.0 + (line_thickness + line_spacing) * i as f32;
            let y = self.body.pos.y - 12.0;

            if i >= self.bullets {
                draw_line(x, y, x, y - line_height, line_thickness, full_color);
            } else {
                draw_line(x, y, x, y - line_height, line_thickness, empty_color);
            };
        }
    }

    pub fn throw(&mut self, force: bool) {
        self.thrown = true;

        if force {
            self.body.speed = if self.body.facing {
                vec2(600., -200.)
            } else {
                vec2(-600., -200.)
            };
        } else {
            self.body.angle = 3.5;
        }

        let mut resources = storage::get_mut::<Resources>();

        let sword_mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + sword_mount_pos,
                40,
                30,
            ));
        } else {
            resources
                .collision_world
                .set_actor_position(self.body.collider.unwrap(), self.body.pos + sword_mount_pos);
        }
        self.origin_pos = self.body.pos + sword_mount_pos / 2.;
    }

    pub fn shoot(node: Handle<MachineGun>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let node = scene::get_node(node);
                if node.bullets <= 0 {
                    let player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);

                    return;
                }
            }

            // {
            //     scene::find_node_by_type::<crate::nodes::Camera>()
            //         .unwrap()
            //         .shake();
            // }
            {
                let resources = storage::get_mut::<Resources>();
                play_sound_once(resources.shoot_sound);

                let mut node = &mut *scene::get_node(node);
                let player = &mut *scene::get_node(player);

                node.fx_active = true;

                let mut bullets = scene::find_node_by_type::<crate::nodes::Bullets>().unwrap();
                bullets.spawn_bullet(node.body.pos, 2.0, node.body.facing);
                player.body.speed.x = -Self::GUN_THROWBACK * player.body.facing_dir();
            }
            {
                let node = &mut *scene::get_node(node);
                node.sprite.set_animation(1);
            }
            for i in 0u32..3 {
                {
                    let node = &mut *scene::get_node(node);
                    node.sprite.set_frame(i);
                    node.fx_sprite.set_frame(i);
                }

                wait_seconds(MachineGun::FIRE_INTERVAL / 3.0).await;
            }
            {
                let mut node = scene::get_node(node);
                node.sprite.set_animation(0);
            }

            {
                let mut node = scene::get_node(node);
                node.fx_active = false;
                node.bullets -= 1;
            }

            {
                let player = &mut *scene::get_node(player);
                // node.weapon_animation.play(0, 0..5).await;
                // node.weapon_animation.play(0, 5..).await;
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<MachineGun>();

            MachineGun::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<MachineGun>()
                .handle();

            MachineGun::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<MachineGun>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<MachineGun>();

            node.body.angle = 0.;
            node.bullets = MachineGun::MAX_BULLETS;
            node.thrown = false;
        }

        capabilities::Gun {
            throw,
            shoot,
            is_thrown,
            pick_up,
        }
    }
}

impl Node for MachineGun {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            Self::gun_capabilities(),
        ));
        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(48.0, 32.0),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();

            if (node.origin_pos - node.body.pos).length() > 70. {
                node.deadly_dangerous = true;
            }
            if node.body.speed.length() <= 200.0 {
                node.deadly_dangerous = false;
            }
            if node.body.on_ground {
                node.deadly_dangerous = false;
            }

            if node.deadly_dangerous {
                let others = scene::find_nodes_by_type::<crate::nodes::Player>();
                let sword_hit_box = Rect::new(node.body.pos.x - 10., node.body.pos.y, 60., 30.);

                for mut other in others {
                    if Rect::new(other.body.pos.x, other.body.pos.y, 20., 64.)
                        .overlaps(&sword_hit_box)
                    {
                        other.kill(!node.body.facing);
                    }
                }
            }
        }
    }

    fn draw(mut node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let mount_pos = if node.thrown == false {
            if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-60., 16.)
            }
        } else {
            if node.body.facing {
                vec2(-25., 0.)
            } else {
                vec2(5., 0.)
            }
        };

        draw_texture_ex(
            resources.machine_gun,
            node.body.pos.x + mount_pos.x,
            node.body.pos.y + mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(node.sprite.frame().dest_size),
                flip_x: !node.body.facing,
                rotation: node.body.angle,
                ..Default::default()
            },
        );

        if node.fx_active {
            node.fx_sprite.update();
            draw_texture_ex(
                resources.machine_gun,
                node.body.pos.x + mount_pos.x,
                node.body.pos.y + mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(node.fx_sprite.frame().source_rect),
                    dest_size: Some(node.fx_sprite.frame().dest_size),
                    flip_x: !node.body.facing,
                    ..Default::default()
                },
            );
        }

        // if let Some(collider) = node.collider {
        //     let pos = resources.collision_world.actor_pos(collider);

        //     let sword_hit_box = Rect::new(pos.x, pos.y, 40., 30.);
        //     draw_rectangle(
        //         sword_hit_box.x,
        //         sword_hit_box.y,
        //         sword_hit_box.w,
        //         sword_hit_box.h,
        //         RED,
        //     );
        // }

        if node.thrown == false {
            node.draw_hud();
        }
    }
}