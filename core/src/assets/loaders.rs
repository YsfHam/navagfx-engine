use crate::{application::GraphicsContextRef, assets::{texture::{RawRgbaImageData, Texture2D}, AssetsLoader}};

pub struct Texture2DLoader {
    context: GraphicsContextRef<'static>
}


impl Texture2DLoader {
    pub(crate) fn new(context: GraphicsContextRef<'static>) -> Self {
        Self {
            context
        }
    }
}

impl AssetsLoader<&str> for Texture2DLoader {
    type TAsset = Texture2D;

    type Error = std::io::Error;

    fn load(&self, file_path: &str) -> std::result::Result<Self::TAsset, Self::Error> {
        let image = image::ImageReader::open(file_path)?
            .decode()
            .unwrap()
            .to_rgba8()
            ;
        
        let context = self.context.read().unwrap();
        Ok(Texture2D::from_image(&context, file_path, &image))
    }
}

impl<'a> AssetsLoader<RawRgbaImageData<'a>> for Texture2DLoader {
    type TAsset = Texture2D;

    type Error = std::io::Error;

    fn load(&self, image: RawRgbaImageData<'a>) -> std::result::Result<Self::TAsset, Self::Error> {
        Ok(
            Texture2D::from_memory(
                &self.context.read().unwrap(),
                "Raw Rgba texture",
                image.pixels,
                image.width,
                image.height
            )
        )
    }
}