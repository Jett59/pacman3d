use bevy::{
    ecs::{component::Component, system::EntityCommands},
    prelude::{
        shape, Assets, BuildChildren, Color, Commands, ComputedVisibility, GlobalTransform,
        PbrBundle, Quat, ResMut, Transform, Vec3, Visibility,
    },
};
use bevy_rapier3d::prelude::{Collider, RigidBody, Velocity};

#[derive(Clone, Default, Debug, Component)]
pub struct GameObject {
    meshes: Vec<Mesh>,
}

impl GameObject {
    pub fn add_mesh(&mut self, mesh: Mesh) -> &mut Self {
        self.meshes.push(mesh);
        self
    }

    pub fn spawn<'w, 's, 'a>(
        self,
        initial_transform: Transform,
        rigid_body: RigidBody,
        commands: &'a mut Commands<'w, 's>,
        meshes: &mut ResMut<Assets<bevy::prelude::Mesh>>,
        materials: &mut ResMut<Assets<bevy::prelude::StandardMaterial>>,
    ) -> EntityCommands<'w, 's, 'a> {
        let colliders = self
            .meshes
            .iter()
            .map(|mesh| (mesh.position, mesh.rotation, mesh.get_collider()))
            .collect::<Vec<_>>();
        let mut children = Vec::with_capacity(self.meshes.len());
        for mesh in self.meshes.iter() {
            children.push(mesh.to_entity(commands, meshes, materials).id());
        }
        let mut entity_commands = commands.spawn(rigid_body);
        children.into_iter().for_each(|child| {
            entity_commands.add_child(child);
        });
        entity_commands
            .insert(Collider::compound(colliders))
            .insert(initial_transform)
            .insert(GlobalTransform::default())
            .insert(Visibility::default())
            .insert(ComputedVisibility::default())
            .insert(Velocity::default());
        entity_commands.insert(self);
        entity_commands
    }
}

#[derive(Clone, Debug)]
pub enum Shape {
    Box { width: f32, height: f32, depth: f32 },
    Sphere { radius: f32 },
    Cylinder { radius: f32, height: f32 },
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub shape: Shape,
    pub color: Color,
    pub position: Vec3,
    pub rotation: Quat,
}

impl Mesh {
    fn to_entity<'w, 's, 'a>(
        &self,
        commands: &'a mut Commands<'w, 's>,
        meshes: &mut ResMut<Assets<bevy::prelude::Mesh>>,
        materials: &mut ResMut<Assets<bevy::prelude::StandardMaterial>>,
    ) -> EntityCommands<'w, 's, 'a> {
        commands.spawn(PbrBundle {
            mesh: meshes.add(match self.shape {
                Shape::Box {
                    width,
                    height,
                    depth,
                } => shape::Box::new(width, height, depth).into(),
                Shape::Cylinder { radius, height } => shape::Cylinder {
                    height,
                    radius,
                    ..Default::default()
                }
                .into(),
                Shape::Sphere { radius } => shape::UVSphere {
                    radius,
                    ..Default::default()
                }
                .into(),
            }),
            material: materials.add(self.color.into()),
            transform: Transform::from_translation(self.position).with_rotation(self.rotation),
            ..Default::default()
        })
    }

    fn get_collider(&self) -> Collider {
        match self.shape {
            Shape::Box {
                width,
                height,
                depth,
            } => Collider::cuboid(width / 2.0, height / 2.0, depth / 2.0),
            Shape::Cylinder { radius, height } => Collider::cylinder(height / 2.0, radius),
            Shape::Sphere { radius } => Collider::ball(radius),
        }
    }
}
