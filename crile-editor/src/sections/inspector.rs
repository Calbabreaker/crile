use crile::AssetPath;

use crate::{project::Project, EditorState, Selection};

pub fn show(ui: &mut egui::Ui, state: &mut EditorState) {
    ui.add_space(5.);

    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
        if let Selection::Entity(id) = state.selection {
            if let Some(meta) = state.scene.world.get::<crile::MetaDataComponent>(id) {
                ui.text_edit_singleline(&mut meta.name);
                ui.add_space(5.);

                let mut entity = state.scene.world.entity_mut(id).unwrap();
                inspect_entity(ui, &mut entity)
            } else {
                state.selection = Selection::None;
            }
        }
    });
}

pub fn update_assets(state: &mut EditorState, engine: &mut crile::Engine) {
    for (_, (sprite,)) in state.scene.world.query_mut::<(crile::SpriteComponent,)>() {
        update_asset(
            &mut sprite.texture,
            &mut sprite.texture_path,
            engine,
            &state.project,
        );
    }

    for (_, (script,)) in state.scene.world.query_mut::<(crile::ScriptComponent,)>() {
        update_asset(
            &mut script.script,
            &mut script.script_path,
            engine,
            &state.project,
        );
    }
}

fn update_asset<Asset: crile::Asset>(
    asset: &mut Option<crile::RefId<Asset>>,
    asset_path: &mut AssetPath,
    engine: &mut crile::Engine,
    project: &Project,
) {
    if asset_path.open_picker {
        asset_path.path = project.pick_file_relative(Asset::PRETTY_NAME, Asset::FILE_EXTENSIONS);
        *asset = None;
        asset_path.open_picker = false;
    }

    if let Some(path) = &asset_path.path {
        if asset.is_none() {
            let absolute_path = project.make_absolute(path);
            let loaded_asset = engine.load_asset(&absolute_path);

            if loaded_asset.is_some() {
                *asset = loaded_asset;
            } else {
                asset_path.path = None;
            }
        }
    }
}

fn inspect_entity(ui: &mut egui::Ui, entity: &mut crile::EntityMut) {
    macro_rules! inspect_components {
        ( [$($component: ty),*]) => {{
            $( inspect_component::<$component>(ui, entity); )*
        }};
    }

    crile::with_components!(inspect_components);

    ui.reset_style();
    ui.separator();

    ui.vertical_centered(|ui| {
        ui.menu_button("Add component", |ui| {
            macro_rules! add_component_buttons {
                ( [$($component: ty),*]) => {{
                    $( add_component_button::<$component>(ui, entity); )*
                }};
            }

            crile::with_components!(add_component_buttons);
        });
    });
}

fn inspect_component<T: Inspectable + crile::Component>(
    ui: &mut egui::Ui,
    entity: &mut crile::EntityMut,
) {
    if let Some(component) = entity.get::<T>() {
        let pretty_name = get_pretty_name::<T>();
        ui.visuals_mut().collapsing_header_frame = true;
        ui.visuals_mut().widgets.noninteractive.bg_stroke.width = 0.;

        let response = egui::CollapsingHeader::new(pretty_name)
            .default_open(true)
            .show(ui, |ui| {
                // TODO: figure out how to align all component drop downs the same
                egui::Grid::new(pretty_name)
                    .num_columns(2)
                    .spacing([30.0, 4.0])
                    .show(ui, |ui| component.inspect(ui));
            });

        response.header_response.context_menu(move |ui| {
            if ui.button("Remove component").clicked() {
                entity.remove::<T>();
                ui.close_menu();
            }
        });
    }
}

fn add_component_button<T: Inspectable + crile::Component>(
    ui: &mut egui::Ui,
    entity: &mut crile::EntityMut,
) {
    if !entity.has::<T>() && ui.button(get_pretty_name::<T>()).clicked() {
        entity.add(T::default());
        ui.close_menu();
    }
}

fn get_pretty_name<T: 'static>() -> &'static str {
    crile::last_type_name::<T>()
        .split("Component")
        .next()
        .unwrap_or_default()
}

pub trait Inspectable {
    fn inspect(&mut self, ui: &mut egui::Ui);
}

impl Inspectable for crile::TransformComponent {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.label("Translation");
        crile_egui::inspect_vec3(ui, &mut self.translation);
        ui.end_row();

        ui.label("Rotation");
        crile_egui::inspect_vec3(ui, &mut self.rotation);
        ui.end_row();

        ui.label("Scale");
        crile_egui::inspect_vec3(ui, &mut self.scale);
        ui.end_row();
    }
}

impl Inspectable for crile::SpriteComponent {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.label("Color");
        crile_egui::inspect_color(ui, &mut self.color);
        ui.end_row();

        ui.label("Texture");
        crile_egui::inspect_asset_path(ui, &mut self.texture_path);
    }
}

impl Inspectable for crile::ScriptComponent {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.label("Script");
        crile_egui::inspect_asset_path(ui, &mut self.script_path);
    }
}

impl Inspectable for crile::CameraComponent {
    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.label("Near");
        crile_egui::inspect_f32(ui, &mut self.near);
        ui.end_row();

        ui.label("Far");
        crile_egui::inspect_f32(ui, &mut self.far);
        ui.end_row();

        match self.projection {
            crile::ProjectionKind::Orthographic => {
                ui.label("Zoom");
                crile_egui::inspect_f32(ui, &mut self.orthographic_zoom);
            }
            crile::ProjectionKind::Perspective => {
                ui.label("Fov");
                crile_egui::inspect_f32(ui, &mut self.perspective_fov);
            }
        }
        ui.end_row();

        ui.label("Projection");
        egui::ComboBox::from_id_source("Projection")
            .selected_text(format!("{:?}", self.projection))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.projection,
                    crile::ProjectionKind::Perspective,
                    "Perspective",
                );
                ui.selectable_value(
                    &mut self.projection,
                    crile::ProjectionKind::Orthographic,
                    "Orthographic",
                );
            });
        ui.end_row();
    }
}
