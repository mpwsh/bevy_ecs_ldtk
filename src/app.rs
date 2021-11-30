//! Types and traits for hooking into the ldtk loading process via bevy's [App].
//!
//! *Requires the "app" feature, which is enabled by default*
use crate::{assets::TilesetMap, components::IntGridCell, ldtk::EntityInstance};
use bevy::{ecs::system::EntityCommands, prelude::*};
use std::{collections::HashMap, marker::PhantomData};

/// Provides a constructor to a bevy [Bundle] which can be used for spawning entities from an LDtk
/// file.
/// After implementing this trait on a bundle, you can register it to spawn automatically for a
/// given identifier via [app.register_ldtk_entity()](RegisterLdtkObjects::register_ldtk_entity).
///
/// For common use cases, you'll want to use derive-macro `#[derive(LdtkEntity)]`, but you can also
/// provide a custom implementation.
///
/// If there is an entity in the LDtk file that is NOT registered, an entity will be spawned with
/// an [EntityInstance] component, allowing you to flesh it out in your own system.
///
/// *Requires the "app" feature, which is enabled by default*
///
/// *Derive macro requires the "derive" feature, which is also enabled by default*
///
/// ## Derive macro usage
/// Using `#[derive(LdtkEntity)]` on a [Bundle] struct will allow the type to be registered to the
/// app via [app.register_ldtk_entity()](RegisterLdtkObjects::register_ldtk_entity):
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_ecs_ldtk::prelude::*;
///
/// fn main() {
///     App::empty()
///         .add_plugin(LdtkPlugin)
///         .register_ldtk_entity::<MyBundle>("my_entity_identifier")
///         // add other systems, plugins, resources...
///         .run();
/// }
///
/// # #[derive(Component, Default)]
/// # struct ComponentA;
/// # #[derive(Component, Default)]
/// # struct ComponentB;
/// # #[derive(Component, Default)]
/// # struct ComponentC;
/// #[derive(Bundle, LdtkEntity)]
/// pub struct MyBundle {
///     a: ComponentA,
///     b: ComponentB,
///     c: ComponentC,
/// }
/// ```
/// Now, when loading your ldtk file, any entities with the entity identifier
/// "my_entity_identifier" will be spawned as `MyBundle`s.
///
/// By default, each component or nested bundle in the bundle will be created using their [Default]
/// implementations.
/// However, this behavior can be overriden with some field attribute macros...
///
/// ### `#[sprite_bundle...]`
/// Indicates that a [SpriteBundle] field should be created with an actual material/image.
/// There are two forms for this attribute:
/// - `#[sprite_bundle("path/to/asset.png")]` will create the field using the image at the provided
/// path in the assets folder.
/// - `#[sprite_bundle]` will create the field using its Editor Visual image in LDtk, if it has one.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecs_ldtk::prelude::*;
/// # #[derive(Component, Default)]
/// # struct Sellable;
/// # #[derive(Component, Default)]
/// # struct PlayerComponent;
/// # #[derive(Component, Default)]
/// # struct Health;
/// #[derive(Bundle, LdtkEntity)]
/// pub struct Gem {
///     #[sprite_bundle("textures/gem.png")]
///     #[bundle]
///     sprite_bundle: SpriteBundle,
///     sellable: Sellable,
/// }
///
/// #[derive(Bundle, LdtkEntity)]
/// pub struct Player {
///     player: PlayerComponent,
///     health: Health,
///     #[sprite_bundle] // Uses the Editor Visual sprite in LDtk
///     #[bundle]
///     sprite_bundle: SpriteBundle,
/// }
/// ```
///
/// ### `#[sprite_sheet_bundle...]`
/// Similar to `#[sprite_bundle...]`, indicates that a [SpriteSheetBundle] field should be created
/// with an actual material/image.
/// There are two forms for this attribute:
/// - `#[sprite_sheet_bundle("path/to/asset.png", tile_width, tile_height, columns, rows, index)]`
/// will create the field using all of the information provided.
/// Similar to using [TextureAtlas::from_grid()].
/// - `#[sprite_sheet_bundle(columns, rows)]` will create the field mostly using information from
/// the LDtk Editor visual, if it has one.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecs_ldtk::prelude::*;
/// # #[derive(Component, Default)]
/// # struct Damage;
/// # #[derive(Component, Default)]
/// # struct BleedDamage;
/// #[derive(Bundle, LdtkEntity)]
/// pub struct Sword {
///     #[bundle]
///     #[sprite_sheet_bundle("weapons.png", 32.0, 32.0, 4, 5, 17)]
///     sprite_sheet: SpriteSheetBundle,
///     damage: Damage,
/// }
///
/// #[derive(Bundle, LdtkEntity)]
/// pub struct Dagger {
///     damage: Damage,
///     bleed_damage: BleedDamage,
///     #[bundle]
///     #[sprite_sheet_bundle(4, 5)]
///     sprite_sheet: SpriteSheetBundle,
/// }
/// ```
///
/// ### `#[ldtk_entity]`
/// Indicates that a nested bundle that implements [LdtkEntity] should be created with
/// [LdtkEntity::bundle_entity], allowing for nested [LdtkEntity]s.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecs_ldtk::prelude::*;
/// # #[derive(Component, Default)]
/// # struct Damage;
/// # #[derive(Component, Default)]
/// # struct BleedDamage;
/// #[derive(Bundle, LdtkEntity)]
/// pub struct Weapon {
///     damage: Damage,
///     #[sprite_bundle]
///     #[bundle]
///     sprite: SpriteBundle,
/// }
///
/// #[derive(Bundle, LdtkEntity)]
/// pub struct Dagger {
///     #[ldtk_entity]
///     #[bundle]
///     weapon_bundle: Weapon,
///     bleed_damage: BleedDamage,
/// }
/// ```
///
/// ### `#[from_entity_instance]`
/// Indicates that a component or bundle that implements [From<EntityInstance>] should be created
/// using that conversion.
/// This allows for more modular and custom component construction, and for different structs that
/// contain the same component to have different constructions of that component, without having to
/// `impl LdtkEntity` for both of them.
/// It also allows you to have an [EntityInstance] field, since all types `T` implement `From<T>`.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecs_ldtk::prelude::*;
/// # #[derive(Component, Default)]
/// # struct Sellable { value: i32 }
/// impl From<EntityInstance> for Sellable {
///     fn from(entity_instance: EntityInstance) -> Sellable {
///         let sell_value = match entity_instance.identifier.as_str() {
///             "gem" => 1000,
///             "nickel" => 5,
///             _ => 10,
///         };
///
///         Sellable {
///             value: sell_value,
///         }
///     }
/// }
///
/// #[derive(Bundle, LdtkEntity)]
/// pub struct NickelBundle {
///     #[sprite_bundle]
///     #[bundle]
///     sprite: SpriteBundle,
///     #[from_entity_instance]
///     sellable: Sellable,
///     #[from_entity_instance]
///     entity_instance: EntityInstance,
/// }
/// ```
pub trait LdtkEntity: Bundle {
    /// The constructor used by the plugin when spawning entities from an LDtk file.
    /// Has access to resources/assets most commonly used for spawning 2d objects.
    /// If you need access to more of the [World], you can create a system that queries for
    /// `Added<EntityInstance>`, and flesh out the entity from there, instead of implementing this
    /// trait.
    /// This is because the plugin spawns an entity with an [EntityInstance] component if it's not
    /// registered to the app.
    ///
    /// Note: whether or not the entity is registered to the app, the plugin will insert [Transform],
    /// [GlobalTransform], and [Parent] components to the entity **after** this bundle is inserted.
    /// So, any custom implementations of these components within this trait will be overwritten.
    fn bundle_entity(
        entity_instance: &EntityInstance,
        tileset_map: &TilesetMap,
        asset_server: &AssetServer,
        materials: &mut Assets<ColorMaterial>,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self;
}

impl LdtkEntity for SpriteBundle {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        tileset_map: &TilesetMap,
        _: &AssetServer,
        materials: &mut Assets<ColorMaterial>,
        _: &mut Assets<TextureAtlas>,
    ) -> Self {
        let tile = match entity_instance.tile.as_ref() {
            Some(tile) => tile,
            None => {
                warn!("#[sprite_bundle] attribute expected the EntityInstance to have a tile defined.");
                return SpriteBundle::default();
            }
        };

        let tileset = match tileset_map.get(&tile.tileset_uid) {
            Some(tileset) => tileset.clone(),
            None => {
                warn!("EntityInstance's tileset should be in the TilesetMap");
                return SpriteBundle::default();
            }
        };

        let material = materials.add(tileset.into());
        SpriteBundle {
            material,
            ..Default::default()
        }
    }
}

pub struct PhantomLdtkEntity<B: LdtkEntity> {
    ldtk_entity: PhantomData<B>,
}

pub trait PhantomLdtkEntityTrait {
    fn evaluate<'w, 's, 'a, 'b>(
        &self,
        commands: &'b mut EntityCommands<'w, 's, 'a>,
        entity_instance: &EntityInstance,
        tileset_map: &TilesetMap,
        asset_server: &AssetServer,
        materials: &mut Assets<ColorMaterial>,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> &'b mut EntityCommands<'w, 's, 'a>;
}

impl<B: LdtkEntity> PhantomLdtkEntityTrait for PhantomLdtkEntity<B> {
    fn evaluate<'w, 's, 'a, 'b>(
        &self,
        entity_commands: &'b mut EntityCommands<'w, 's, 'a>,
        entity_instance: &EntityInstance,
        tileset_map: &TilesetMap,
        asset_server: &AssetServer,
        materials: &mut Assets<ColorMaterial>,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> &'b mut EntityCommands<'w, 's, 'a> {
        entity_commands.insert_bundle(B::bundle_entity(
            entity_instance,
            tileset_map,
            asset_server,
            materials,
            texture_atlases,
        ))
    }
}

/// Used by [RegisterLdtkObjects] to associate Ldtk entity identifiers with [LdtkEntity]s.
pub type LdtkEntityMap = HashMap<String, Box<dyn PhantomLdtkEntityTrait>>;

/// Provides a constructor to a bevy [Bundle] which can be used for spawning additional components
/// on IntGrid tiles.
/// After implementing this trait on a bundle, you can register it to spawn automatically for a
/// given int grid value via
/// [app.register_ldtk_int_cell()](RegisterLdtkObjects::register_ldtk_int_cell).
///
/// For common use cases, you'll want to use derive-macro `#[derive(LdtkIntCell)]`, but you can
/// also provide a custom implementation.
///
/// If there is an IntGrid tile in the LDtk file whose value is NOT registered, an entity will be
/// spawned with an [IntGridCell] component, allowing you to flesh it out in your own system.
///
/// *Requires the "app" feature, which is enabled by default*
///
/// *Derive macro requires the "derive" feature, which is also enabled by default*
///
/// ## Derive macro usage
/// Using `#[derive(LdtkIntCell)]` on a [Bundle] struct will allow the type to be registered to the
/// app via [app.register_ldtk_int_cell()](RegisterLdtkObjects::register_ldtk_int_cell):
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_ecs_ldtk::prelude::*;
///
/// fn main() {
///     App::empty()
///         .add_plugin(LdtkPlugin)
///         .register_ldtk_int_cell::<MyBundle>(1)
///         // add other systems, plugins, resources...
///         .run();
/// }
///
/// # #[derive(Component, Default)]
/// # struct ComponentA;
/// # #[derive(Component, Default)]
/// # struct ComponentB;
/// # #[derive(Component, Default)]
/// # struct ComponentC;
/// #[derive(Bundle, LdtkIntCell)]
/// pub struct MyBundle {
///     a: ComponentA,
///     b: ComponentB,
///     c: ComponentC,
/// }
/// ```
/// Now, when loading your ldtk file, any IntGrid tiles with the value `1` will be spawned with as
/// tiles with `MyBundle` inserted.
///
/// By default, each component or nested bundle in the bundle will be created using their [Default]
/// implementations.
/// However, this behavior can be overriden with some field attribute macros...
///
/// ### `#[ldtk_int_cell]`
/// Indicates that a nested bundle that implements [LdtkIntCell] should be created with
/// [LdtkIntCell::bundle_int_cell], allowing for nested [LdtkIntCell]s.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecs_ldtk::prelude::*;
/// # #[derive(Component, Default)]
/// # struct RigidBody;
/// # #[derive(Component, Default)]
/// # struct Damage;
/// #[derive(Bundle, LdtkIntCell)]
/// pub struct Wall {
///     rigid_body: RigidBody,
/// }
///
/// #[derive(Bundle, LdtkIntCell)]
/// pub struct DestructibleWall {
///     #[ldtk_int_cell]
///     #[bundle]
///     wall: Wall,
///     damage: Damage,
/// }
/// ```
///
/// ### `#[from_int_grid_cell]`
/// Indicates that a component or bundle that implements [From<IntGridCell>] should be created
/// using that conversion.
/// This allows for more modular and custom component construction, and for different structs that
/// contain the same component to have different constructions of that component, without having to
/// `impl LdtkIntCell` for both of them.
/// It also allows you to have an [IntGridCell] field, since all types `T` implement `From<T>`.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecs_ldtk::prelude::*;
/// # #[derive(Component, Default)]
/// # struct Fluid { viscosity: i32 }
/// # #[derive(Component, Default)]
/// # struct Damage;
/// impl From<IntGridCell> for Fluid {
///     fn from(int_grid_cell: IntGridCell) -> Fluid {
///         let viscosity = match int_grid_cell.value {
///             1 => 5,
///             2 => 20,
///             _ => 0,
///         };
///
///         Fluid {
///             viscosity,
///         }
///     }
/// }
///
/// #[derive(Bundle, LdtkIntCell)]
/// pub struct Lava {
///     #[from_int_grid_cell]
///     fluid: Fluid,
///     #[from_int_grid_cell]
///     int_grid_cell: IntGridCell,
///     damage: Damage,
/// }
/// ```
pub trait LdtkIntCell: Bundle {
    /// The constructor used by the plugin when spawning additional components on IntGrid tiles.
    /// If you need access to more of the [World], you can create a system that queries for
    /// `Added<IntGridCell>`, and flesh out the entity from there, instead of implementing this
    /// trait.
    /// This is because the plugin spawns a tile with an [IntGridCell] component if the tile's
    /// value is not registered to the app.
    ///
    /// Note: whether or not the entity is registered to the app, the plugin will insert [Transform],
    /// [GlobalTransform], and [Parent] components to the entity **after** this bundle is inserted.
    /// So, any custom implementations of these components within this trait will be overwritten.
    /// Furthermore, a [bevy_ecs_tilemap::TileBundle] will be inserted **before** this bundle, so
    /// be careful not to overwrite the components provided by that bundle.
    fn bundle_int_cell(int_grid_cell: IntGridCell) -> Self;
}

pub struct PhantomLdtkIntCell<B: LdtkIntCell> {
    ldtk_int_cell: PhantomData<B>,
}

pub trait PhantomLdtkIntCellTrait {
    fn evaluate<'w, 's, 'a, 'b>(
        &self,
        entity_commands: &'b mut EntityCommands<'w, 's, 'a>,
        int_grid_cell: IntGridCell,
    ) -> &'b mut EntityCommands<'w, 's, 'a>;
}

impl<B: LdtkIntCell> PhantomLdtkIntCellTrait for PhantomLdtkIntCell<B> {
    fn evaluate<'w, 's, 'a, 'b>(
        &self,
        entity_commands: &'b mut EntityCommands<'w, 's, 'a>,
        int_grid_cell: IntGridCell,
    ) -> &'b mut EntityCommands<'w, 's, 'a> {
        entity_commands.insert_bundle(B::bundle_int_cell(int_grid_cell))
    }
}

pub type LdtkIntCellMap = HashMap<i32, Box<dyn PhantomLdtkIntCellTrait>>;

/// Provides the [.register_ldtk_entity()](RegisterLdtkObjects::register_ldtk_entity) and
/// [.register_ldtk_int_cell()](RegisterLdtkObjects::register_ldtk_int_cell) function to bevy's
/// [App].
///
/// Not intended for custom implementations on your own types, but you're still welcome to do so.
///
/// *Requires the "app" feature, which is enabled by default*
pub trait RegisterLdtkObjects {
    /// Registers [LdtkEntity] types to be spawned for a given Entity identifier in an LDtk file.
    ///
    /// This example lets the plugin know that it should spawn a MyBundle when it encounters a
    /// "my_entity_identifier" entity in an LDtk file.
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_ecs_ldtk::prelude::*;
    ///
    /// fn main() {
    ///     App::empty()
    ///         .add_plugin(LdtkPlugin)
    ///         .register_ldtk_entity::<MyBundle>("my_entity_identifier")
    ///         // add other systems, plugins, resources...
    ///         .run();
    /// }
    ///
    /// # #[derive(Component, Default)]
    /// # struct ComponentA;
    /// # #[derive(Component, Default)]
    /// # struct ComponentB;
    /// # #[derive(Component, Default)]
    /// # struct ComponentC;
    /// #[derive(Bundle, LdtkEntity)]
    /// pub struct MyBundle {
    ///     a: ComponentA,
    ///     b: ComponentB,
    ///     c: ComponentC,
    /// }
    /// ```
    ///
    /// You can find more details on the `#[derive(LdtkEntity)]` macro at [LdtkEntity].
    fn register_ldtk_entity<B: LdtkEntity>(&mut self, identifier: &str) -> &mut Self;

    /// Registers [LdtkIntCell] types to be inserted for a given IntGrid value in an LDtk file.
    ///
    /// This example lets the plugin know that it should spawn a MyBundle when it encounters an
    /// IntGrid tile whose value is `1`.
    /// ```no_run
    /// use bevy::prelude::*;
    /// use bevy_ecs_ldtk::prelude::*;
    ///
    /// fn main() {
    ///     App::empty()
    ///         .add_plugin(LdtkPlugin)
    ///         .register_ldtk_int_cell::<MyBundle>(1)
    ///         // add other systems, plugins, resources...
    ///         .run();
    /// }
    ///
    /// # #[derive(Component, Default)]
    /// # struct ComponentA;
    /// # #[derive(Component, Default)]
    /// # struct ComponentB;
    /// # #[derive(Component, Default)]
    /// # struct ComponentC;
    /// #[derive(Bundle, LdtkIntCell)]
    /// pub struct MyBundle {
    ///     a: ComponentA,
    ///     b: ComponentB,
    ///     c: ComponentC,
    /// }
    /// ```
    fn register_ldtk_int_cell<B: LdtkIntCell>(&mut self, value: i32) -> &mut Self;
}

impl RegisterLdtkObjects for App {
    fn register_ldtk_entity<B: LdtkEntity>(&mut self, identifier: &str) -> &mut App {
        let new_entry = Box::new(PhantomLdtkEntity::<B> {
            ldtk_entity: PhantomData,
        });
        match self.world.get_non_send_resource_mut::<LdtkEntityMap>() {
            Some(mut entries) => {
                entries.insert(identifier.to_string(), new_entry);
            }
            None => {
                let mut bundle_map = LdtkEntityMap::new();
                bundle_map.insert(identifier.to_string(), new_entry);
                self.world.insert_non_send::<LdtkEntityMap>(bundle_map);
            }
        }
        self
    }

    fn register_ldtk_int_cell<B: LdtkIntCell>(&mut self, value: i32) -> &mut Self {
        let new_entry = Box::new(PhantomLdtkIntCell::<B> {
            ldtk_int_cell: PhantomData,
        });
        match self.world.get_non_send_resource_mut::<LdtkIntCellMap>() {
            Some(mut entries) => {
                entries.insert(value, new_entry);
            }
            None => {
                let mut bundle_map = LdtkIntCellMap::new();
                bundle_map.insert(value, new_entry);
                self.world.insert_non_send::<LdtkIntCellMap>(bundle_map);
            }
        }
        self
    }
}
