content:
    - title: Septem Modi
      content: !RoleList
      - role: Game code
        name: InnocentusLime
      - role: Textures
        name: InnocentusLime and RiksM
    - title: Game engine
      content: !Subsections
      - title: About the used modules
        content: !Verbatim |-
          All modules mentioned are Rust crates, which can be found on crates.io, all extra
          information regarding the modules can be found on their docs.rs page and the linked github.
      - title: Bevy engine
        content: !RoleList
        - role: engine
          name: cart, bevyengine team
      - title: Third party plugins for bevy
        content: !RoleList
        - role: Bevy ecs tilemap
          name: StarArawn
        - role: Bevy mod debugdump
          name: jakobhellermann
        - role: iyes loopless
          name: inodentry (IyesGames)
        - role: Bevy asset loader
          name: NiklasEi
        - role: Bevy common asserts
          name: NiklasEi
        - role: Bevy inspector egui
          name: jakobhellermann
      - title: Other dependencies
        content: !RoleList
        - role: serde
          name: dtolnay, serde-rs team
        - role: thiserror
          name: dtolnay
        - role: anyhow
          name: dtolnay
        - role: tiled
          name: mattyhall, bjorn and aleokdev
        - role: clap
          name: kbknapp, clap-rs team
        - role: image
          name: theotherphil, fintelia, HeroicKatora, image-rs team
    - title: Special thanks
      content: !Verbatim |-
        Special thanks to my family, all the contributors to the bevy and its thirdparty plugins and
        the bevy and rust communities.