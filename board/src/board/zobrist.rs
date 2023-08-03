use crate::board::constants::{ColorBits, PieceBits, QUEEN, SquareShiftBits, ZobristHash};

pub enum Zobrist {}

impl Zobrist {
    const PIECE_SQUARE_HASHES: [[ZobristHash; 64]; 14] = [
        [0; 64],
        [0x53f955076a9af49b, 0xd583265f12ce1f81, 0x1474e049bbc32904, 0x5f15ae2ea589007e, 0xc0e37ad279f86405, 0x798cfaac3428e82c, 0x1969dea02c9f623a, 0xbe2613412fe80b61, 0xfe743e204188d50b, 0x3d17e08c3371fc86, 0xcccbbd19b7eb28c6, 0xb489c04c21851515, 0x11f14ca1cd8d2542, 0x43c88c1b97b802c6, 0xc0515190ca461ee9, 0x1693e617b0a64427, 0x53ccde4b7cd60cb2, 0x5d42b6c190e98b8e, 0x3e6997fea6d3bf68, 0xaba8cc9615354648, 0xcc2df5b7d0dd9ff5, 0x1651f9cb48a4600c, 0x38d2e4f142a7b010, 0x1cc24e05ae5ca4a7, 0x38c6ba61167e10e0, 0xd5f6bbfd6c63a4d0, 0x233da1873e218fa, 0x8c6c0d1887a3d096, 0xe1ab6c8b62f502d2, 0x30b2e44432b85a0d, 0x5863c4efccdcd4ef, 0xdf2572cb9d5ed392, 0x9a413c4bdf2cb7f4, 0x4079ee1f4b704258, 0xfb563922e0eaa523, 0x4e847b106e01472c, 0x9aa53e6b77f48391, 0x67e9eaa364e58117, 0x5db235b48a7b3969, 0xcf31d003e18941da, 0x1a651c071cd7949d, 0x7bd0fb52e1557159, 0xf25b93807d0780b5, 0xa9f1a5f25f426336, 0x93dbfbb5cbea3ca7, 0x368f794471e49f61, 0xed87d79281ec3d65, 0x76004f3c73c45640, 0xa5a052f13e02067, 0xe5c23be688adb0d, 0x82eb289f7ef63112, 0xf3ec9391d5bac1e, 0xb75f213f54210c11, 0x2e80fb6140858c9f, 0x9b8743ea637e7264, 0x908788676a4f0e28, 0xe873a4de469caef8, 0xe85222f36a5a70cc, 0xd7efdcc594b449c3, 0x8d255af1b41a6d18, 0xb65a22de8f50e046, 0xc29a7554e8762895, 0x9c15f63f6b266df0, 0xc93de0a847167f27, ],
        [0x8e074b5bc16ff63d, 0xdf310e859a966411, 0xb7bf279e2a8d08f3, 0xdabbcbef1c629e71, 0xb7320b7eeba8ea35, 0x8cf9d4d91996a52b, 0xa7aeb43928c3bce1, 0x61e19a516046b462, 0xf88f230af244bb6b, 0x8ce614020777941, 0x78f22876f21df632, 0xe6ffce7bad205843, 0xece954f0ef08f725, 0xb46ad2be7599a5ff, 0x2d53727c9f73d489, 0x9026b476cdfbf1f4, 0xb3296e774ddab3c0, 0x1efc2b9d34e1df7e, 0xa9f65a6ff9f22524, 0x1cc6eb978696e5f3, 0xc8176c51d316c4a, 0xf5779bb35c802af1, 0x93809faa2ea6e550, 0x8f6e36c475d65b16, 0xb7eac8a0681dc92e, 0xb7cedf11e27801d7, 0xa52d1630f80c507b, 0x3f3ec8c8ebd427c8, 0xda33b56c3df9d243, 0x1bcaf8a972518fe7, 0x3310f14bc58c7725, 0x757c22d2393d9c30, 0xd8e86fe434843253, 0x1f42f87c26a93956, 0xb6dc61fa7a5421ca, 0xfe49ed3643281b5f, 0x2a5b2984c5c5a5ce, 0xcb91e20ac81d67c8, 0x67de9bbc69f99ea8, 0x7e83a2e6debad59b, 0xf6af380d0812e14e, 0xabaaee7a6caf55a, 0xf4d471887b847a96, 0xd1a8b5740fa7d242, 0x66f2c4f353805bae, 0x76cecf5a6650ce00, 0xd48a6b735b170ae6, 0x4b5fde094ca21238, 0xfe8030ec7be786c8, 0xdbb0d75b7052fc1d, 0xd18a2dd79a7c784, 0xb10752898eb2e062, 0xdb858b9352e626e1, 0xed4dfff86f2b85b9, 0x3834b2e408690f8d, 0xf6ddc694eb8d9ea9, 0xa8468b8c14e75d35, 0x65f51e85832b8b9f, 0xe85cd435f9b56c61, 0xb33e7a6ca1db4f23, 0xbf7fdc75dbcef33e, 0x93d2e425e9853ba9, 0xf961a2c71b6ce7e1, 0x3f10fac3add9257a, ],
        [0x943d582794571241, 0xcc514dbaf95f5bc8, 0x68bc24be13450aef, 0x2b1a3193081579a0, 0x7098b31575cb989f, 0xe9ca10716a82e724, 0x2808954d939e1c2b, 0x9eb063a052331937, 0x773cf8b26629f020, 0xb1b406dd5762938c, 0x9b68931379129632, 0xa50cfee31fac8e8a, 0x4f49d2223a96d84d, 0x2edbbd39d3497ce6, 0xac0a60f39137357a, 0xe04ffca7b4029ba9, 0x938bd54e5283791e, 0xfed47c1f9f6d4ffa, 0x626951d179f769ba, 0x241abab0283f7b6c, 0x525b0daf862da4cf, 0xf9efea5b612badde, 0x4653dc849143f928, 0xbda3c58ed789d985, 0x9c218cfa4ae56683, 0xd2e120465a411a41, 0x3434464ee5e90cc3, 0x4f4e005c4a14f84d, 0xdf04d2f7dfb73ca8, 0x6cbf7adf3b2979f7, 0x5bbd1c0811fa2daa, 0xfe86f41cfe85f83b, 0xe86b2fafcc872a11, 0xc07109846bba6b4b, 0x1c6602f9fce64a4f, 0xc468cd3fb3dec44b, 0xf7e2c4f699ce13be, 0x369dd06b7aa15ee3, 0xc7f2178df3ea9e51, 0xf749ebad8a7ac8a3, 0x61505258cb64084c, 0xbb416fa61bcfc56, 0x9c993464a6cd6d68, 0x18464852f66eec64, 0xf0fe6e87372e414a, 0x75797460ae34f6b, 0x9db7c7126a19263d, 0xa489e460f5645297, 0x3be8f8685b370735, 0x48c3e70e9e5996c4, 0xc68344de8c7ed81, 0xe9e45a53a3da249d, 0x95d770d2e35cf596, 0x514ce987fb939830, 0x8e13b2fdbf04c5a0, 0xd465a51e547c5864, 0xf7b9d2e0f14f5f91, 0xceb2550de57284e0, 0xc83717c04761e4f6, 0x173accd17d99012, 0x6a06ff9e109271ed, 0x691c2eba82b47bf9, 0x8f03095bd359ba5e, 0xfe10af42946886f0, ],
        [0x2d25464b11957946, 0xb10803a67e1b6b5c, 0x6ed56c8b3e4a2748, 0x4511157c81997589, 0x5dcbdb5b95954200, 0xe56991740dc092e, 0xc472be0fd065e8f0, 0x3af8e1bc156465c0, 0xf902217ecab6838e, 0x310ff3f266816cd3, 0xb68cf2595f7a44ab, 0x5bd0b8075cecbad6, 0xa44c78e3dd7271b0, 0xed12226bae295258, 0x710e921c9fbcd8da, 0x1ee3e34e7f9d78da, 0x50ababb4fdb58342, 0x612ce75bd3681d14, 0xd1dcedfa0b87d226, 0x7a493eeae351bd35, 0x3ad822d24205a419, 0x118949b128f03958, 0x44868cd8ae648513, 0x78e2d7e1fa8de3e3, 0xa0b222ca5167762b, 0xcce046a63e88aa2a, 0x81a10cb3bce9af12, 0x2927baec5edd9e04, 0x67bbeeb2a59655e5, 0xebfbf288474a8759, 0xb9aa98d2ce96b81, 0xe0b7382363f715d7, 0x4d44b40438e5004e, 0xcc290613bfebbd99, 0x7dcab63ee1632023, 0x404561866b56bb9, 0x82f7171646e61bc1, 0xf05a1daf2a98aa38, 0xcc1394d5a444be89, 0x28c63cbebbbf2775, 0xb527c19005b695b9, 0x6c672dd305cbd903, 0x91533e24f47ac40a, 0x57af79c78649315a, 0xb276fc027398af4a, 0xbc7c2b6e4eb205e2, 0x1e6983cb5d623145, 0x286b96eee3f95780, 0xa92e009c0243581b, 0x1d71a8a93a142200, 0xa5b4ab45e5941833, 0x7710fb5fe86a6ece, 0xf6503abedd30e012, 0x89b476ff55c43ea9, 0x8942ee478a8af6e0, 0x182015d61d907b48, 0xa803e56c6a154469, 0x8f8ae0632ac38130, 0x9204ea3506fb2517, 0x7ebfab3a9ac34719, 0xc5743695e918e5ea, 0xe6c8bcf977645db8, 0x6fd1d32cc2aa3e08, 0xccff3862b1966b10, ],
        [0xae10680d5edebb0c, 0xec61703f9e10ff6e, 0xc671bc59c44dd3c9, 0x4297145f379356b, 0xfd6c9bd01c2be238, 0xcf9b249e9ba4dc, 0xe4aa3c3c1d2debe, 0xe7508fbbbf25b371, 0x8fd55e468c24433e, 0xc3ed6a56945c4234, 0xb76f0c887e62121f, 0x65bd34e6d48cba92, 0xbb9308026117c604, 0x356b18b9fd07ef1a, 0x57969663d4fc4b6b, 0x4f93aeaf4659672e, 0x438c5e29c551d0a2, 0x80e35d3d752662ec, 0x289c9785e7a6a6d8, 0xfec8c12bc1073144, 0x55727797d6aa5544, 0xd152e6264c58ee25, 0xc95284e8ca7f33c4, 0x8c2b3425c19423ce, 0xec0e45418823dfa0, 0xbb6702396c35026c, 0x71ee60d8e52bc555, 0x9762e0bb35f98818, 0x492530b089feeb1a, 0x488d7c32839208e8, 0x1cdd147dd2635987, 0x86d6d2821eae0462, 0xe0cc57e329e1bb33, 0x2ccd60d6e4f104e1, 0xcb1d7276dd6e694f, 0xbc0c971a32354fb7, 0xafd370ecd15f9c80, 0xa0dc3e33e3fea34f, 0x84bcfb4d9dbb0a2f, 0x22b61d67ace2d41a, 0x126c24c510565f42, 0x2bfa12f22b915c37, 0x74ec3d324d2b3dbb, 0x3b07fc3b0cda8d47, 0x37af85b1032fa501, 0xccac71b1442ea0dc, 0xdcd7b7ac80e58aca, 0x7cf51c270b85a494, 0x521660a4e85a2d22, 0x4e21867b86962e78, 0x3a952a8556cdfbb1, 0x22d64a79480f27e4, 0x70c5a416d0d8e119, 0x4f0d8d6e3a1af892, 0x980d50db457fc95f, 0x2d301addcf482125, 0xe2dc472f7840c384, 0x456cff6ad34a5e73, 0x1d0e6f734b6ce9b9, 0xfa03b37faeb227e8, 0x4f19f1b48e74b153, 0x9496be78b5f4c185, 0x2dbeb978d1d7ce42, 0x2928901dc221549a, ],
        [0x3d52d491f8ddccd5, 0x9b4afe6200307e67, 0x3a77523cf0c0509b, 0x1e27c3ab36e88a5c, 0x4b2f727d5e55a6e4, 0xae4d74484df8e8bf, 0x5918877ef784da82, 0xa0624d81b5ffa1c0, 0xed036000ba30cd8e, 0x833ff04b264fa1de, 0x4978cc7f6812fc29, 0x9091af7c2bc16104, 0xe0c4e9ed615e263b, 0x58c1453dd208fca4, 0x4158119f435fd86e, 0x93db4d1b5ba3409b, 0x85f134c4a462b81b, 0xd05d83df1327304c, 0xa7052535bb462dce, 0xae5282e741ea8174, 0xa63accf08130d4f7, 0xdbdc6dee70815b2, 0xf3d486236df4ce23, 0x8d7a226e4e6085a0, 0x97aac9354393c0bc, 0xcd5804ad16d1d56b, 0x86efa847db17da61, 0x50a154b3a08e4826, 0xd29ceed876b53a13, 0x8b0e5701bd47c311, 0xfe1c954b95621c2d, 0x58c9622327a0a2e7, 0xef0e4956057fba75, 0x18d4821285007e87, 0x905fec7cc016e60a, 0xfbcd48f29a1db36d, 0x3f844a9c9c0afe38, 0xae82cd3c8b4e7737, 0x6d97f8be03d4f58f, 0xf53771bde5d72d06, 0xea5b1da60edce6a7, 0xca292620b9158d11, 0xc45ad326b3a66165, 0xd7b5d2dd8304b368, 0xb6e7d08752b71456, 0xd7c5fbe738815efe, 0x725e91acb210623f, 0xb2d4dd2caa50027a, 0x2508364d3872ed56, 0x4cba05890dc325a6, 0x6fa827da3911f1e9, 0xc587b9e131cdc676, 0xc899ac0d9d504343, 0x42a90b1ffea32919, 0xc3bc8f932ec278bc, 0x8023af42072afc7d, 0x973916416ecf62f7, 0x35f729aa0951e604, 0x48399d18af6fb729, 0xd7ccf8f46d9dcd71, 0xc31542d1d6d2f0b9, 0x5938a2f77295ec20, 0xa88c548393e49865, 0x35b917dacfbcc7a1, ],
        [0; 64],
        [0xc797ac119113ea5d, 0x309da3f2ef02e423, 0xecf547a0cca357b8, 0xfda625603c6e2aac, 0x21d85594ac2ecf6b, 0xc4e6e26625c3c10a, 0xee8f702eab8d4fe4, 0xcecc4f78e03b36fd, 0x92c34ff5d7e2e82a, 0xacf80231bf16b820, 0x4980f633eb090212, 0xf107edc20444b1a, 0xf5cd6bd633dc5b2e, 0x8b2edb027ba8e4f4, 0x954224a38be3a4dc, 0x7858aabfc3526296, 0x5307e8df63911be6, 0x95c6fdacb6a4e4a3, 0xa0738570cc7cf8a, 0xc8fadf087abe3bea, 0x82aa9f0848a14186, 0xeb5004fd5db8d970, 0x7fcffc5cd0b26451, 0xb00537ca9a2f5cf7, 0x290506fb1bda136a, 0xde73487d69654288, 0x5b4ddfe05492fcd1, 0x6ff9de8dcdbc69b, 0x81a41e5174c22de4, 0xd55fea3c683fdacd, 0x6112df4043dab836, 0x44c9f62dd907476, 0xac7e2302c702013c, 0x480dc68906b4c331, 0x364ba2e0c57b158b, 0xae0b6a2c8fc0f272, 0xffedd6e4b976a208, 0x65f2f522181c04eb, 0xf42a40449c66fa9c, 0xd5441f95a14358cb, 0xfcd0232970e4697d, 0x8cf580734361ebcf, 0xc28c87af70c9cf70, 0xfb2f011b7d73b08a, 0xd2115b5435819e9c, 0x61cdd349c0b3082d, 0x6920adc8696443b3, 0xc627d8fb3324213c, 0x7b75968b16e749c, 0x9df308b7b9630c3f, 0xcab41252a84449d0, 0x62420129d97cb06e, 0x4cd2027e5ee9b9d7, 0xa7cb1ce7bbda3b89, 0xca75e4ef25bf10bf, 0xd54eedf795b2c7d4, 0xad8f4eda428b653a, 0xce4c12196cc3ce3d, 0x734b8c87883a3eac, 0x27b5f153a6197fd4, 0x52f9ebf032608951, 0x2b94736a0eda3919, 0x4c813f0bd50b7788, 0xdc91c3c64b8b95, ],
        [0xdb5e27241c4ae6ea, 0x22e447d03144524d, 0xa7f2dcd207eecb60, 0x4fdb8453913f85b6, 0x9b775e9675307081, 0x1af0d4484c20c7d0, 0xbdcc8ea6b5bdf8b7, 0xbb84cf4c2fc343d0, 0xdd9731dc8bb6848e, 0x13a1505722d8a789, 0xb1d99b86c904df9d, 0x9e53b13f2c3742fd, 0xc0da680c17b56bc9, 0xa8e720f96d0a6052, 0xbbbfa34ce127e9a1, 0xb9e92681bbf5396b, 0x74f0b94a248d62b, 0xfca52461e3ef05fe, 0x40d68c5babb48c3c, 0x94a9a7bbe253d7ce, 0x9130d8ebb4fdd75e, 0xd833fb4457c1251d, 0xd423bd25d758139e, 0x4c0b41164a50d016, 0x623c1a570f3eed6d, 0xcebdbd1fff4052d7, 0x5c79e3b1ce692dde, 0x984f6ad0fa465154, 0x47aafd9e7b10dadc, 0x29fe9bb755e42ecd, 0x8ff2b4005a5692b3, 0x4c7ba6d0bca40957, 0x400cff32787fd1a9, 0xcce7e54c15a11768, 0x6e3659354b8729ac, 0x206e97dc7f66e12c, 0xd81b373eebaeda1a, 0x3b033680198fe689, 0xae2b2fd0b0495b29, 0x14410bee3a290324, 0xef90f7d4cd6c3bc1, 0x155ed66a8aa4fe4c, 0xf46a34e26819c4f5, 0x258314cdd7094dfa, 0xa18157847f8e42ed, 0x2a86a33b8d4507d0, 0x1753ca736ee3f053, 0x3210744f1c95e451, 0x390329e3da748216, 0xf1309c6e18e6cbd6, 0x4342d083aa85e00, 0x2ac06751582f9b10, 0xcadf51bbbc4f5764, 0x66d8afa11a2cc773, 0xd1a95f580f153aed, 0x39a69fc1532a19b2, 0x32a4e0cdc9e638b, 0x39240b37c0c529bf, 0x48739265d50977, 0x6019f2061f146fe2, 0x886891b83acf1ae6, 0x1489264dfbb7c10d, 0xb63ee6eaa1e1a35e, 0x74b1c9e27646fe64, ],
        [0x6dfbb9056c4a8ae7, 0x39c8fbb1816fdb63, 0x9bd95590fd5fb658, 0xebd3f3c42199f826, 0xd656867f469266e3, 0x347c141a6a4aea94, 0x9b30f31ee5ed1b45, 0x51c5b220754c76f8, 0xee1fdb0f8d27ac9a, 0xe12feebf30879dbf, 0x358a25542c2bf61b, 0x8ed0df2b40a1b89b, 0x38c4c66bd2f7e4b2, 0xd9f51fae1ee1a58e, 0xa2901588723dcae0, 0xa44ec61401e73b95, 0x6f3bd2f273a2c96d, 0xc6b582c3586ad87b, 0xecd92b26291980cf, 0x1087a4c1bd201f10, 0xfee4302e32884d4b, 0x898643c1de062027, 0xfb3ddcd51dbfa9ee, 0xfd94a8339393566c, 0x74870979f35d6bb5, 0x78462416f66c103, 0x7b3dc0ea2ff4c350, 0x62c245bcb45ef346, 0xcd386c76e631ebcf, 0x903fcbe0ba9a5c53, 0x97d9716fb8a3e80b, 0x237690b692391d24, 0x3427efd0eed54e3b, 0xe60c66a6915cda4d, 0xc76181c1354f6b8b, 0x1a69c6c74754dc60, 0x8f464ad86143314c, 0xfeb566f9695d9944, 0x7cc84e6249f48022, 0x219962a563a6753f, 0x7f41a6dd9c60ab8c, 0xfd6bf8f2ca635c6e, 0x39184253e251dbd0, 0xb1dedee316a1fc1c, 0xf29be5f3db718159, 0x8c9b388bc365ed2e, 0xae1dc5cd07aaadd0, 0x63a1cc9ed88f435f, 0x33b5fa4ea3dfcd1c, 0x44361485acdb1fd7, 0xee44362be659bf60, 0xab402d4def454e14, 0x17b28e88949f2979, 0xc271cbdb08111237, 0x95ce721420bd43b0, 0x946f4cc86860ff74, 0x2c60cf667b144c35, 0x654b05b25d6244c5, 0x57d3d235fc5da9a5, 0x956a0f0d67e62541, 0x8c0dd4a1245f90da, 0x2ff4283ff8b049dd, 0x573dbafe083b4c4, 0x426ecd184e5b4332, ],
        [0xeff2d76c3a7f54d7, 0x84ce98aaadbd40c7, 0x85591ee12a4cb16b, 0x49cd4c04c0144e80, 0x104dcde3d961c3e5, 0x85a29ab3b2c4738c, 0xdebeeecaa7c60d5c, 0x76b4aa8b98f7f6f1, 0x8277af1a0a6b5c7f, 0xe030c93d4de85c4e, 0x1a7090b1ae955bc7, 0xc74185ffb1b3e6e7, 0x7b75d4db83b964d3, 0x42058d82781d8cd6, 0x76a7c00a0cb3bf6, 0x39da72f6e01e39d1, 0x49d292c0d1c4f32d, 0xde6de41f08dc2492, 0xe214c57d8d03f77f, 0x694560ca7cffa8bf, 0x665e6b0503612395, 0x1777fdccc04645a, 0x516c37144cb3015e, 0x8fc1221e713f0e7a, 0xb013ea2e99352fee, 0x8b3eb82369d9fadd, 0x53ea8744a8c9da71, 0x4edacb44a177fb74, 0xdf3adb89ae751f46, 0x1ed6044651a056c9, 0xc30a449a67883ef6, 0x8ac50459693f3b58, 0xc4b1b5b97e8d7424, 0x1eb93b4b9998291c, 0x8c1c28e6e664738a, 0x5eceaecd48249cd4, 0x1eae441e0aaadd6e, 0xf80050f608ae5402, 0xcdcca8db5bb6465, 0xc470c3bcaf9ee221, 0x349763b85989cae9, 0xa5c356dcd04e138b, 0x6d519ae186d333ee, 0xba02b6418c4eb229, 0x5e6d25ee7919dc44, 0xdd3cb37c5fffa6c4, 0x27d4de93eebbcdf3, 0x2ee9b1bbe45eb188, 0xf7fda00fde222b68, 0xad2cc9b73500f269, 0xdd66626b4526c642, 0xc266ea294c3169c5, 0x2d43d86940cfa929, 0x70a1a4dcce62562f, 0x3937ef2f4c339c7d, 0x58ad947414f781f3, 0x13e8e7f84022f9c3, 0xf32481f3e8ed3270, 0xe07d6eb7caba366f, 0xa9303eaba3e45128, 0xa42a3587e50d4441, 0x2fc2c21282d4cd83, 0xcba024d1838ad606, 0x5487c5815e71f551, ],
        [0xf03bde7cc0ab78ff, 0x8ca226776b478350, 0x48cd131798fc7824, 0x4f174f6ab4e90b58, 0x5e58f6312e9a693, 0x61bea92986326b38, 0xd0987cbaea18befc, 0x53eb8b6e2a5a3a6d, 0x5c9e45cb44a4be0c, 0x69bc8cd36fd54513, 0x10550dfeca708edb, 0x13e17c14e6d54776, 0xc52a501e26d290b8, 0xee61036be777d1a7, 0xac271b5e25eba825, 0xede77d43da8aa35d, 0xa9663132b80d1950, 0xef3f7f859c05c7cd, 0x84695df3f6204995, 0x4ab18eb84216887e, 0xacf0120060fa28fc, 0x4aa321cdaeae4fff, 0xf6dd7f0a69f8a7a0, 0x266db25e54bdbefc, 0x5cf60ae3507198a9, 0xb2c2f5b2ab82d09c, 0xdaaf103255067ff2, 0xbbfd845c1c017a52, 0xb16e37ee72041fdc, 0x5054395a380104a3, 0x26c4b53a4353d443, 0x23a8c30fa0b60b7b, 0x802c62e896972350, 0x76cb28b2ee43b0df, 0x119a251b44ea1de2, 0x946132972a0810a4, 0xd78a7c4a421ea3c5, 0x3387510a8f0acfb3, 0xf7538d9acd2dbf3d, 0x383e94078451b0dc, 0x1b44e84272f8e9c1, 0x2ab3df4d6763ef9c, 0x40f451be7b88116b, 0xcb1c8df9daadcdca, 0xe89b527cff9d5845, 0xc4a5a4f7551e809a, 0xd44d0791e08adb7a, 0x202bed31070c3df6, 0x7605d4081e51d776, 0x795310934991eb53, 0x55051b38f9d3374d, 0x4ee2e23428b82d36, 0xae915f7fd2a03bc, 0x2086ac262f9f7ac3, 0x9fe40003e497a55, 0x4f15cc61cc14c19a, 0x521998debc2272f1, 0x85f827aecaaadf0e, 0x623f430a9a54b242, 0x27e0d34a9d38882e, 0x4d248fbbbe97ace1, 0x62d7ec2afcc79d77, 0x86e759dd111aa6b2, 0x61bcf45a56af545d, ],
        [0x860e61fae166827d, 0x893639ccae83a94, 0x2f8fd2f050410c0e, 0xf4673108fc4f9869, 0xa45743a5128fbbe7, 0xa7c9cf003f2c5149, 0xb4c9b879f55969f2, 0x633b414ca3d10bbe, 0x10f5e10f6189b6a, 0xfaeb1c9a7460491e, 0xe2ff96f580c20e3a, 0xc3e1a70750ec39c8, 0x6dee4612c16c3a01, 0xeccb2508f34f5ff8, 0x117d638bd390dc64, 0x46dc7fef1a97be36, 0xdf187b8df090cdbf, 0x8ebe7b8aba0c1fee, 0xb9f320824348351a, 0x7cc16f4667d848de, 0xed5316505adaf5d9, 0x19f5e096114c2b4e, 0x3e690110aef6e8a1, 0x36a9dd17cb68788a, 0x28a00af7464a0c8e, 0x34dba7562c79506f, 0xc08507998d9bb0c3, 0x465e693a7a766563, 0xa6287cb3fe6fb02e, 0x71db7ebca6483810, 0x64571f12c7e018a9, 0xfd9edb5f2fd1f04e, 0x215f10389a630d2f, 0xe0d43c733c5a67ce, 0x192050aa4eb9a39e, 0xa670ab69c5270cb, 0x82327406621d7560, 0x27524580f247f088, 0x3b8cf6070f8f1b4e, 0xc0dd2a48b64c4284, 0xfb888490333e8d9f, 0x3d0c84f8c37a06f7, 0xe902eeccae045a66, 0x9c1bd1df6aaf3029, 0x7c35badaba0c32b1, 0xc32e7d73af599463, 0x30615a94e12db3ce, 0x3327fdd9435f97f9, 0x22cb2ccab980925d, 0xeeaef640f9cb5d5a, 0xed0ec7e1ab2bd6f4, 0x37492d99361af276, 0x6e134eee7227b418, 0xa50d129fb612a6bf, 0xb18d45e0590a04ba, 0x749769d39027330b, 0x1be7225776bab58e, 0x171cbb2f2f491887, 0xa01de922bcabfb45, 0xf8dee4681b15c544, 0xd66b55fc41bedbe1, 0x4a1a2ec836a02417, 0xab4db6700ec0f114, 0x168a36dc3dab8189, ],
    ];
    const EN_PASSANT_HASHES: [ZobristHash; 8] = [
        0x585f7318b3b13343, 0x7e3277d482da39fd, 0x52d9b5dbd5a51800, 0xdf7a013672f81767, 0xddca97f3c265524e, 0x68e368c97a813450, 0x52ec396777bf0452, 0x8a496b857337906d,
    ];
    pub const WHITE_KING_CASTLE_HASH: ZobristHash = 0x7e376d4dc831dc15;
    pub const WHITE_QUEEN_CASTLE_HASH: ZobristHash = 0xcb3046dc8c54f87d;
    pub const BLACK_KING_CASTLE_HASH: ZobristHash = 0x2cd08e6b1034a445;
    pub const BLACK_QUEEN_CASTLE_HASH: ZobristHash = 0xe6d1cdeac4ec0b8a;
    pub const BLACK_TO_MOVE_HASH: ZobristHash = 0xaa56513abd96ba3;

    const CASTLE_HASHES: [[ZobristHash; 2]; 2] = [
        [Self::WHITE_QUEEN_CASTLE_HASH, Self::WHITE_KING_CASTLE_HASH],
        [Self::BLACK_QUEEN_CASTLE_HASH, Self::BLACK_KING_CASTLE_HASH]
    ];

    #[inline(always)]
    pub const fn castle_hash(side: PieceBits, color: ColorBits) -> ZobristHash {
        Self::CASTLE_HASHES[color as usize][(side - QUEEN) as usize]
    }

    #[inline(always)]
    pub const fn piece_square_hash(piece: PieceBits, square: SquareShiftBits, color: ColorBits) -> ZobristHash {
        Self::PIECE_SQUARE_HASHES[(piece as usize) + (7 * color as usize)][square as usize]
    }

    #[inline(always)]
    pub const fn en_passant_square_hash(square: SquareShiftBits) -> ZobristHash {
        Self::EN_PASSANT_HASHES[(square % 8) as usize]
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    use crate::board::constants::{BLACK, KING, QUEEN, WHITE, ZobristHash};
    use crate::board::zobrist::Zobrist;

    #[test]
    fn test_castle() {
        assert_eq!(Zobrist::WHITE_QUEEN_CASTLE_HASH, Zobrist::castle_hash(QUEEN, WHITE));
        assert_eq!(Zobrist::WHITE_KING_CASTLE_HASH, Zobrist::castle_hash(KING, WHITE));
        assert_eq!(Zobrist::BLACK_QUEEN_CASTLE_HASH, Zobrist::castle_hash(QUEEN, BLACK));
        assert_eq!(Zobrist::BLACK_KING_CASTLE_HASH, Zobrist::castle_hash(KING, BLACK));
    }

    #[test]
    #[ignore]
    fn generate() {
        let mut hashes = HashSet::new();
        let mut rng: StdRng = SeedableRng::from_seed([0; 32]);

        let mut gen_single = || -> ZobristHash {
            loop {
                let next = rng.gen();
                if hashes.insert(next) {
                    return next;
                }
            };
        };

        let mut gen_64 = || -> [ZobristHash; 64] {
            let mut result = [0; 64];
            for i in 0..64 {
                result[i] = gen_single();
            }
            result
        };

        let square_piece_hashes = [
            [0; 64], gen_64(), gen_64(), gen_64(), gen_64(), gen_64(), gen_64(),
            [0; 64], gen_64(), gen_64(), gen_64(), gen_64(), gen_64(), gen_64(),
        ];

        let en_passant_hashes = gen_64();

        let white_king_castle_hash = gen_single();
        let white_queen_castle_hash = gen_single();
        let black_king_castle_hash = gen_single();
        let black_queen_castle_hash = gen_single();

        let black_to_move = gen_single();

        println!("const PIECE_SQUARE_HASHES: [[ZobristHash; 64]; 14] = {:#0x?};", square_piece_hashes);
        println!("const EN_PASSANT_HASHES: [ZobristHash; 8] = {:#0x?};", &en_passant_hashes[..8]);
        println!("pub const WHITE_KING_CASTLE_HASH: ZobristHash = {:#0x?};", white_king_castle_hash);
        println!("pub const WHITE_QUEEN_CASTLE_HASH: ZobristHash = {:#0x?};", white_queen_castle_hash);
        println!("pub const BLACK_KING_CASTLE_HASH: ZobristHash = {:#0x?};", black_king_castle_hash);
        println!("pub const BLACK_QUEEN_CASTLE_HASH: ZobristHash = {:#0x?};", black_queen_castle_hash);
        println!("pub const BLACK_TO_MOVE: ZobristHash = {:#0x?};", black_to_move);
    }
}
