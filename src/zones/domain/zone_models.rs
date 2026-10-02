use crate::models::domain::model::{Model, RemoteFile};

pub const SUBJECT_MODEL: Model = Model {
    name: "subject",
    file: RemoteFile {
        url: "https://huggingface.co/onnx-community/BiRefNet_lite-ONNX/resolve/de15b22ba131738a16dff04aab8bdf8dc32e3ac1/onnx/model.onnx",
        sha256: "5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333",
        size: 224_005_088,
    },
};

pub const SKY_MODEL: Model = Model {
    name: "sky",
    file: RemoteFile {
        url: "https://huggingface.co/JianyuanWang/skyseg/resolve/3ba8c6df1d9ba9ff26f637c7ba9568ac11a9aa7f/skyseg.onnx",
        sha256: "ab9c34c64c3d821220a2886a4a06da4642ffa14d5b30e8d5339056a089aa1d39",
        size: 175_997_079,
    },
};

pub const PERSONS_MODEL: Model = Model {
    name: "persons",
    file: RemoteFile {
        url: "https://huggingface.co/onnx-community/rfdetr_small-ONNX/resolve/63463b68b200177d1fea7015f11f3cebb0ba4eeb/onnx/model.onnx",
        sha256: "121cc1476a7b69d865ca4bdc2bca59a3239020d227e4c3f35a811bef81aeb7f1",
        size: 114_680_416,
    },
};

pub const BODY_PARTS_MODEL: Model = Model {
    name: "body parts",
    file: RemoteFile {
        url: "https://huggingface.co/Xenova/segformer_b2_clothes/resolve/cb6ac44e641faa309f29561c1afe08d87ef52631/onnx/model.onnx",
        sha256: "12aee5578c95cd43706062d9a37e7359b320c147c92c437681c730a580716ef4",
        size: 110_039_291,
    },
};

pub const FACE_PARTS_MODEL: Model = Model {
    name: "face parts",
    file: RemoteFile {
        url: "https://huggingface.co/Xenova/face-parsing/resolve/f25b9b521a8783d4e78e80e026ef4c2a15f821e0/onnx/model.onnx",
        sha256: "6d4e67af60ff78184745ebf74cc15163c0adc27d45cdeba31e3a03d1096fb8c3",
        size: 340_316_611,
    },
};
