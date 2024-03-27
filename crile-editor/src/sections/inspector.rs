use crate::{EditorState, Selection};

pub fn show(state: &mut EditorState, ui: &mut egui::Ui) {
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
        // Open file picker when requested
        if sprite.texture_path.open_picker {
            sprite.texture_path.path = state
                .project
                .pick_file_relative("Image", &["jpeg", "jpg", "png"]);
            sprite.texture = None;
            sprite.texture_path.open_picker = false;
        }

        if let Some(path) = &sprite.texture_path.path {
            if sprite.texture.is_none() {
                let absolute_path = state.project.make_absolute(path);
                let texture = engine
                    .asset_library
                    .load_texture(&engine.gfx.wgpu, &absolute_path);

                if texture.is_some() {
                    sprite.texture = texture;
                } else {
                    sprite.texture_path.path = None;
                }
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

fn inspect_component<T: Inspectable + 'static>(ui: &mut egui::Ui, entity: &mut crile::EntityMut) {
    if let Some(component) = entity.get::<T>() {
        // TODO: figure out how to make header fill available width
        let response = egui::CollapsingHeader::new(T::pretty_name())
            .default_open(true)
            .show_background(true)
            .show_unindented(ui, |ui| {
                ui.visuals_mut().widgets.noninteractive.bg_stroke.width = 0.;
                ui.spacing_mut().indent = 8.;

                ui.indent(T::pretty_name(), |ui| {
                    egui::Grid::new(T::pretty_name())
                        .num_columns(2)
                        .spacing([30.0, 4.0])
                        .show(ui, |ui| component.inspect(ui));
                });
            });

        response.header_response.context_menu(move |ui| {
            if ui.button("Remove component").clicked() {
                entity.remove::<T>();
                ui.close_menu();
            }
        });
    }
}

fn add_component_button<T: Inspectable + Default + 'static>(
    ui: &mut egui::Ui,
    entity: &mut crile::EntityMut,
) {
    if !entity.has::<T>() && ui.button(T::pretty_name()).clicked() {
        entity.add(T::default());
        ui.close_menu();
    }
}

pub trait Inspectable {
    fn inspect(&mut self, ui: &mut egui::Ui);
    fn pretty_name() -> &'static str;
}

impl Inspectable for crile::TransformComponent {
    fn pretty_name() -> &'static str {
        "Transform Component"
    }

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
    fn pretty_name() -> &'static str {
        "Sprite Component"
    }

    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.label("Color");
        crile_egui::inspect_color(ui, &mut self.color);
        ui.end_row();

        ui.label("Texture");
        if ui.button("Choose file").clicked() {
            self.texture_path.open_picker = true;
        }
    }
}

impl Inspectable for crile::CameraComponent {
    fn pretty_name() -> &'static str {
        "Camera Component"
    }

    fn inspect(&mut self, ui: &mut egui::Ui) {
        ui.label("Near");
        crile_egui::inspect_f32(ui, &mut self.near);
        ui.end_row();

        ui.label("Far");
        crile_egui::inspect_f32(ui, &mut self.far);
        ui.end_row();

        match self.projection_kind {
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
            .selected_text(format!("{:?}", self.projection_kind))
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.projection_kind,
                    crile::ProjectionKind::Perspective,
                    "Perspective",
                );
                ui.selectable_value(
                    &mut self.projection_kind,
                    crile::ProjectionKind::Orthographic,
                    "Orthographic",
                );
            });
        ui.end_row();
    }
}
