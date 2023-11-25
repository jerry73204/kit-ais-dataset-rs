use anyhow::{format_err, Result};
use clap::Parser;
use kit_ais_dataset::Dataset;
use opencv::{core as core_cv, highgui, imgcodecs, imgproc};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
/// Visualize KIT AIS data images with bounding boxes.
struct Opts {
    /// Data set directory.
    pub dataset_dir: PathBuf,
    /// Output directory to save the image files.
    pub output_dir: PathBuf,
    #[structopt(long)]
    /// Disable GUI.
    pub no_gui: bool,
}

fn main() -> Result<()> {
    let Opts {
        dataset_dir,
        output_dir,
        no_gui,
    } = Opts::parse();

    fs::create_dir_all(&output_dir)?;

    fs::read_dir(dataset_dir)?
        .map(|entry| -> Result<_> {
            let entry = entry?;
            let path = entry.file_type()?.is_dir().then(|| entry.path());
            Ok(path)
        })
        .filter_map(|path| path.transpose())
        .try_for_each(|sub_dir| -> Result<_> {
            let sub_dir = sub_dir?;
            let xml_path = sub_dir
                .read_dir()?
                .map(|entry| -> Result<_> {
                    let entry = entry?;

                    let path = if entry.file_type()?.is_file() {
                        let file_name = entry.file_name();
                        let file_name: &Path = file_name.as_ref();
                        let extension = file_name.extension().and_then(|ext| ext.to_str());

                        if let Some("xml") = extension {
                            Some(entry.path())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    Ok(path)
                })
                .find_map(|path| path.transpose())
                .transpose()?
                .ok_or_else(|| format_err!("XML file not found"))?;

            let dir_name = sub_dir.file_name().unwrap();

            let text = std::fs::read_to_string(&xml_path)?;
            let dataset: Dataset = serde_xml_rs::from_str(&text)?;

            let output_sub_dir = output_dir.join(dir_name);
            fs::create_dir_all(&output_sub_dir)?;

            dataset.frames.iter().try_for_each(|frame| -> Result<_> {
                let image_path = sub_dir.join(&frame.file);
                let image_name = frame.file.file_name().unwrap();

                eprintln!("processing '{}'", image_path.display());

                let mut image = imgcodecs::imread(image_path.to_str().unwrap(), 0)?;
                imgproc::cvt_color(&image.clone(), &mut image, imgproc::COLOR_GRAY2BGR, 0)?;

                frame
                    .object_list
                    .objects
                    .iter()
                    .try_for_each(|obj| -> Result<_> {
                        let r#box = &obj.r#box;
                        let rect = core_cv::Rect {
                            x: (r#box.xc - r#box.w / 2.0).raw() as i32,
                            y: (r#box.yc - r#box.h / 2.0).raw() as i32,
                            width: r#box.w.raw() as i32,
                            height: r#box.h.raw() as i32,
                        };
                        let color = core_cv::Scalar::new(0.0, 255.0, 0.0, 0.0);
                        imgproc::rectangle(&mut image, rect, color, 2, imgproc::LINE_8, 0)?;

                        Ok(())
                    })?;

                let output_image_path = output_sub_dir.join(image_name);
                imgcodecs::imwrite(
                    output_image_path.to_str().unwrap(),
                    &image,
                    &core_cv::Vector::new(),
                )?;

                if !no_gui {
                    highgui::imshow("viewer", &image)?;
                    highgui::wait_key(1)?;
                }

                Ok(())
            })?;

            Ok(())
        })?;

    Ok(())
}
