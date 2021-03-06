use id3::{
    frame::{Comment, EncapsulatedObject, ExtendedLink, ExtendedText, Lyrics, PictureType},
    Frame, Tag, TagLike,
};
use neon::{prelude::*, types::buffer::TypedArray};

fn u8_vec_to_arraybuffer<'a, C: Context<'a>>(
    cx: &mut C,
    vec: &Vec<u8>,
) -> JsResult<'a, JsArrayBuffer> {
    let mut buffer = cx.array_buffer(vec.len())?;
    buffer.as_mut_slice(cx).copy_from_slice(vec);
    Ok(buffer)
}

fn arraybuffer_to_u8_vec<'a, C: Context<'a>>(
    cx: &mut C,
    buffer: &Handle<JsArrayBuffer>,
) -> Vec<u8> {
    buffer.as_slice(cx).to_vec()
}

fn tag_to_js_tag<'a, C: Context<'a>>(cx: &mut C, tag: &Tag) -> JsResult<'a, JsArray> {
    let js_tag: Handle<JsArray> = cx.empty_array();

    macro_rules! transfer_frame_as_tuple {
        ($i:expr, $tag_type:expr, $tag_id:expr, $tag_content:expr) => {
            let js_type = cx.string($tag_type);
            let js_key = cx.string($tag_id);
            let js_remove = cx.boolean(false);

            let js_tuple = cx.empty_array();
            js_tuple.set(cx, 0, js_type).unwrap();
            js_tuple.set(cx, 1, js_key).unwrap();
            js_tuple.set(cx, 2, $tag_content).unwrap();
            js_tuple.set(cx, 3, js_remove).unwrap();

            js_tag
                .set(cx, $i, js_tuple)
                .expect("Failed writing a tag to JavaScript object");

            $i += 1;
        };
    }

    // Write frames to js_tag
    let mut i: u32 = 0;
    tag.frames().for_each(|frame| {
        match frame.content() {
            // Texts
            id3::Content::Text(content) => {
                let js_string = cx.string(content);
                transfer_frame_as_tuple!(i, "text", frame.id(), js_string);
                return;
            }

            // Extended texts
            id3::Content::ExtendedText(content) => {
                let js_value = cx.string(&content.value);
                let js_description = cx.string(&content.description);

                let js_extended_text = cx.empty_object();
                js_extended_text
                    .set(cx, "value", js_value)
                    .expect("Failed writing an extended text frame value to Javascript runtime");
                js_extended_text
                    .set(cx, "description", js_description)
                    .expect(
                        "Failed writing an extended text frame description to Javascript runtime",
                    );

                transfer_frame_as_tuple!(i, "extended text", frame.id(), js_extended_text);
                return;
            }

            // Links
            id3::Content::Link(content) => {
                let js_text = cx.string(content);
                transfer_frame_as_tuple!(i, "link", frame.id(), js_text);
                return;
            }

            // Extended links
            id3::Content::ExtendedLink(content) => {
                let js_extended_link = cx.empty_object();
                let js_description = cx.string(&content.description);
                let js_link = cx.string(&content.link);

                js_extended_link
                    .set(cx, "description", js_description)
                    .unwrap();
                js_extended_link.set(cx, "link", js_link).unwrap();

                transfer_frame_as_tuple!(i, "extended link", frame.id(), js_extended_link);
                return;
            }

            // Comments
            id3::Content::Comment(content) => {
                let js_lang = cx.string(&content.lang);
                let js_description = cx.string(&content.description);
                let js_text = cx.string(&content.text);

                let js_comment = cx.empty_object();
                js_comment
                    .set(cx, "lang", js_lang)
                    .expect("Failed writing a comment frame lang to Javascript runtime");
                js_comment
                    .set(cx, "description", js_description)
                    .expect("Failed writing a comment frame description to Javascript runtime");
                js_comment
                    .set(cx, "text", js_text)
                    .expect("Failed writing a comment frame text to Javascript runtime");

                transfer_frame_as_tuple!(i, "comment", frame.id(), js_comment);
                return;
            }

            // Popularimeters
            // id3::Content::Popularimeter(content) => todo!(),

            // Lyrics
            id3::Content::Lyrics(content) => {
                let js_lyrics = cx.empty_object();
                let js_lang = cx.string(&content.lang);
                let js_description = cx.string(&content.description);
                let js_text = cx.string(&content.text);

                js_lyrics.set(cx, "lang", js_lang).unwrap();
                js_lyrics.set(cx, "description", js_description).unwrap();
                js_lyrics.set(cx, "text", js_text).unwrap();

                transfer_frame_as_tuple!(i, "lyrics", frame.id(), js_lyrics);
                return;
            }

            // SynchronisedLyrics
            // id3::Content::SynchronisedLyrics(content) => todo!(),

            // Pictures
            id3::Content::Picture(content) => {
                let js_picture = cx.empty_object();
                let js_mime_type = cx.string(&content.mime_type);
                let js_picture_type = cx.number(u8::from(content.picture_type));
                let js_description = cx.string(&content.description);
                let js_data = u8_vec_to_arraybuffer(cx, &content.data)
                    .expect("Failed loading image data into Javascript runtime");

                js_picture
                    .set(cx, "MIMEType", js_mime_type)
                    .expect("Failed writing a picture frame MIME type to Javascript runtime");
                js_picture
                    .set(cx, "pictureType", js_picture_type)
                    .expect("Failed writing picture frame picture type to Javascript runtime");
                js_picture
                    .set(cx, "description", js_description)
                    .expect("Failed writing picture frame description to Javascript runtime");
                js_picture
                    .set(cx, "data", js_data)
                    .expect("Failed writing picture frame picture data to Javascript runtime");

                transfer_frame_as_tuple!(i, "picture", frame.id(), js_picture);
                return;
            }

            // Encapsulated objects
            id3::Content::EncapsulatedObject(content) => {
                let js_enc_object = cx.empty_object();
                let js_mime_type = cx.string(&content.mime_type);
                let js_filename = cx.string(&content.filename);
                let js_description = cx.string(&content.description);
                let js_data = u8_vec_to_arraybuffer(cx, &content.data)
                    .expect("Failed loading image data into Javascript runtime");

                js_enc_object.set(cx, "MIMEType", js_mime_type).unwrap();
                js_enc_object.set(cx, "filename", js_filename).unwrap();
                js_enc_object
                    .set(cx, "description", js_description)
                    .unwrap();
                js_enc_object.set(cx, "data", js_data).unwrap();

                transfer_frame_as_tuple!(i, "encapsulated object", frame.id(), js_enc_object);
            }

            // Chapters
            // id3::Content::Chapter(content) => todo!(),

            // MpegLocationLookupTables
            // id3::Content::MpegLocationLookupTable(content) => todo!(),

            // Unknown frames
            id3::Content::Unknown(content) => {
                let js_unknown = cx.empty_object();
                // let js_version = cx.string(&content.version);
                let js_data = u8_vec_to_arraybuffer(cx, &content.data).unwrap();

                // js_unknown.set(&mut cx, "version", js_version).unwrap();
                js_unknown.set(cx, "data", js_data).unwrap();

                transfer_frame_as_tuple!(i, "unknown", frame.id(), js_unknown);
            }

            // Frames that are not implemented yet
            _ => {
                panic!("Unsupporeted frame type {}", frame.to_string());
            }
        }
    });

    Ok(js_tag)
}

fn load_tag(mut cx: FunctionContext) -> JsResult<JsArray> {
    let js_path: Handle<JsString> = cx.argument(0).expect("Incorrect argument 0 received");
    let path = js_path.value(&mut cx);

    // Read tag or create a new one
    let tag;
    match Tag::read_from_path(&path) {
        Ok(t) => {
            tag = t;
        }
        Err(error) => match error.kind {
            id3::ErrorKind::NoTag => tag = Tag::new(),
            _ => panic!("Error reading tag: {}", &error.description),
        },
    };

    let js_tag =
        tag_to_js_tag(&mut cx, &tag).expect("Failed converting tag to a javascript tuple array");

    Ok(js_tag)
}

fn u8_to_picture_ype(i: u8) -> PictureType {
    match i {
        1 => PictureType::Icon,
        2 => PictureType::OtherIcon,
        3 => PictureType::CoverFront,
        4 => PictureType::CoverBack,
        5 => PictureType::Leaflet,
        6 => PictureType::Media,
        7 => PictureType::LeadArtist,
        8 => PictureType::Artist,
        9 => PictureType::Conductor,
        10 => PictureType::Band,
        11 => PictureType::Composer,
        12 => PictureType::Lyricist,
        13 => PictureType::RecordingLocation,
        14 => PictureType::DuringRecording,
        15 => PictureType::DuringPerformance,
        16 => PictureType::ScreenCapture,
        17 => PictureType::BrightFish,
        18 => PictureType::Illustration,
        19 => PictureType::BandLogo,
        20 => PictureType::PublisherLogo,
        _ => PictureType::Other,
    }
}

fn update_tag(mut cx: FunctionContext) -> JsResult<JsArray> {
    let js_path: Handle<JsString> = cx.argument(0).expect("Incorrect argument 0 received");
    let path = js_path.value(&mut cx);
    let js_tag: Handle<JsArray> = cx.argument(1).expect("Incorrect argument 1 received");

    // Create a tag instance and try to load tag from path
    let mut tag;
    match Tag::read_from_path(&path) {
        Ok(t) => {
            tag = t;
        }
        Err(error) => match error.kind {
            // File at path has no tag, creat one
            id3::ErrorKind::NoTag => tag = Tag::new(),
            _ => panic!("Error reading tag: {}", &error.description),
        },
    };

    let frame_tuples: Vec<Handle<JsValue>> = js_tag.to_vec(&mut cx).expect("");

    frame_tuples.iter().for_each(|tuple| {
        match tuple.downcast_or_throw::<JsArray, FunctionContext>(&mut cx) {
            Ok(js_tuple) => {
                let js_frame_type: Handle<JsString> = js_tuple.get(&mut cx, 0).unwrap();
                let frame_type = js_frame_type.value(&mut cx);
                let js_frame_name: Handle<JsString> = js_tuple.get(&mut cx, 1).unwrap();
                let frame_name = js_frame_name.value(&mut cx);
                let js_frame_remove: Handle<JsBoolean> = js_tuple.get(&mut cx, 3).unwrap();
                let frame_remove = js_frame_remove.value(&mut cx);

                // Remove or set this frame
                if frame_remove {
                    // Remove pictures
                    if frame_type == "picture" {
                        let js_frame_content: Handle<JsObject> = js_tuple.get(&mut cx, 2).unwrap();
                        let js_picture_type: Handle<JsNumber> = js_frame_content
                            .get(&mut cx, "pictureType")
                            .expect("APIC.pictureType not provided");
                        let picture_type = u8_to_picture_ype(js_picture_type.value(&mut cx) as u8);

                        println!("{}", picture_type);

                        tag.remove_picture_by_type(picture_type);
                        // tag.remove_all_pictures();
                    }
                    // Remove encapsulate objects
                    else if frame_type == "encapsulated object" {
                        let js_frame_content: Handle<JsObject> = js_tuple.get(&mut cx, 2).unwrap();
                        let js_mime_type: Handle<JsString> =
                            js_frame_content.get(&mut cx, "MIMEType").unwrap();
                        let js_filename: Handle<JsString> =
                            js_frame_content.get(&mut cx, "filename").unwrap();
                        let js_description: Handle<JsString> =
                            js_frame_content.get(&mut cx, "description").unwrap();
                        let js_data: Handle<JsArrayBuffer> =
                            js_frame_content.get(&mut cx, "data").unwrap();

                        let mime_type = js_mime_type.value(&mut cx);
                        let filename = js_filename.value(&mut cx);
                        let description = js_description.value(&mut cx);
                        let data = arraybuffer_to_u8_vec(&mut cx, &js_data);

                        tag.remove_encapsulated_object(
                            Some(&description),
                            Some(&mime_type),
                            Some(&filename),
                            Some(&data),
                        );
                    }
                    // Remove other frame types
                    else {
                        tag.remove(frame_name);
                    }
                } else {
                    // Texts
                    if frame_type == "text" {
                        let js_frame_content: Handle<JsString> = js_tuple.get(&mut cx, 2).unwrap();
                        let frame_content = js_frame_content.value(&mut cx);
                        tag.add_frame(Frame::text(frame_name, frame_content));
                    }
                    // Extended texts
                    else if frame_type == "extended text" {
                        let js_frame_content: Handle<JsObject> = js_tuple.get(&mut cx, 2).unwrap();
                        let js_description: Handle<JsString> =
                            js_frame_content.get(&mut cx, "description").unwrap();
                        let js_value: Handle<JsString> =
                            js_frame_content.get(&mut cx, "value").unwrap();

                        tag.add_frame(ExtendedText {
                            description: js_description.value(&mut cx),
                            value: js_value.value(&mut cx),
                        });
                    }
                    // Links
                    else if frame_type == "link" {
                        let js_frame_content: Handle<JsString> = js_tuple.get(&mut cx, 2).unwrap();
                        let frame_content = js_frame_content.value(&mut cx);
                        tag.add_frame(Frame::link(frame_name, frame_content));
                    }
                    // Extended links
                    else if frame_type == "extended link" {
                        let js_frame_content: Handle<JsObject> = js_tuple.get(&mut cx, 2).unwrap();
                        let js_description: Handle<JsString> =
                            js_frame_content.get(&mut cx, "description").unwrap();
                        let js_link: Handle<JsString> =
                            js_frame_content.get(&mut cx, "link").unwrap();

                        tag.add_frame(ExtendedLink {
                            description: js_description.value(&mut cx),
                            link: js_link.value(&mut cx),
                        });
                    }
                    // Lyrics
                    else if frame_type == "lyrics" {
                        let js_frame_content: Handle<JsObject> = js_tuple.get(&mut cx, 2).unwrap();
                        let js_lang: Handle<JsString> =
                            js_frame_content.get(&mut cx, "lang").unwrap();
                        let js_description: Handle<JsString> =
                            js_frame_content.get(&mut cx, "description").unwrap();
                        let js_text: Handle<JsString> =
                            js_frame_content.get(&mut cx, "text").unwrap();

                        tag.add_frame(Lyrics {
                            lang: js_lang.value(&mut cx),
                            description: js_description.value(&mut cx),
                            text: js_text.value(&mut cx),
                        });
                    }
                    // Comments
                    else if frame_type == "comment" {
                        let js_frame_content: Handle<JsObject> = js_tuple.get(&mut cx, 2).unwrap();
                        let js_lang: Handle<JsString> =
                            js_frame_content.get(&mut cx, "lang").unwrap();
                        let js_description: Handle<JsString> =
                            js_frame_content.get(&mut cx, "description").unwrap();
                        let js_text: Handle<JsString> =
                            js_frame_content.get(&mut cx, "text").unwrap();

                        tag.add_frame(Comment {
                            lang: js_lang.value(&mut cx),
                            description: js_description.value(&mut cx),
                            text: js_text.value(&mut cx),
                        });
                    }
                    // Pictures
                    else if frame_type == "picture" {
                        let js_frame_content: Handle<JsObject> = js_tuple.get(&mut cx, 2).unwrap();
                        let js_mime_type: Handle<JsString> = js_frame_content
                            .get(&mut cx, "MIMEType")
                            .expect("APIC.MIMEType not provided");
                        let js_picture_type: Handle<JsNumber> = js_frame_content
                            .get(&mut cx, "pictureType")
                            .expect("APIC.pictureType not provided");
                        let js_description: Handle<JsString> = js_frame_content
                            .get(&mut cx, "description")
                            .expect("APIC.description not provided");
                        let js_data: Handle<JsArrayBuffer> = js_frame_content
                            .get(&mut cx, "data")
                            .expect("APIC.data not provided");

                        let mime_type = js_mime_type.value(&mut cx);
                        let picture_type = u8_to_picture_ype(js_picture_type.value(&mut cx) as u8);
                        let description = js_description.value(&mut cx);
                        let data: Vec<u8> = arraybuffer_to_u8_vec(&mut cx, &js_data);

                        let picture = id3::frame::Picture {
                            mime_type,
                            picture_type,
                            description,
                            data,
                        };
                        tag.add_frame(Frame::with_content(
                            "APIC",
                            id3::Content::Picture(picture.clone()),
                        ));
                    }
                    // Encapsulated object
                    else if frame_type == "encapsulated object" {
                        let js_frame_content: Handle<JsObject> = js_tuple.get(&mut cx, 2).unwrap();
                        let js_mime_type: Handle<JsString> =
                            js_frame_content.get(&mut cx, "MIMEType").unwrap();
                        let js_filename: Handle<JsString> =
                            js_frame_content.get(&mut cx, "filename").unwrap();
                        let js_description: Handle<JsString> =
                            js_frame_content.get(&mut cx, "description").unwrap();
                        let js_data: Handle<JsArrayBuffer> =
                            js_frame_content.get(&mut cx, "data").unwrap();

                        tag.add_frame(EncapsulatedObject {
                            mime_type: js_mime_type.value(&mut cx),
                            filename: js_filename.value(&mut cx),
                            description: js_description.value(&mut cx),
                            data: arraybuffer_to_u8_vec(&mut cx, &js_data),
                        });
                    } else {
                        panic!("Saving frame of type {} is not implemented yet", frame_type);
                    }
                }
            }

            Err(_) => {}
        }
    });

    tag.write_to_path(&path, id3::Version::Id3v24)
        .expect("Failed saving tag to file");

    let js_tag =
        tag_to_js_tag(&mut cx, &tag).expect("Failed converting tag to a javascript tuple array");

    Ok(js_tag)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("loadTag", load_tag)?;
    cx.export_function("updateTag", update_tag)?;
    Ok(())
}
