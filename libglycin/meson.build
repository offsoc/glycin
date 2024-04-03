if get_option('libglycin')
    libglycin_deps = [
        dependency('gio-2.0', version: gio_req),
        dependency('gtk4', version: gtk4_req),
        dependency('libseccomp', version: seccomp_req),
        dependency('lcms2', version: lcms2_req),
    ]

    pkg = import('pkgconfig')
    gnome = import('gnome')

    cargo_options = [
        '--target-dir',
        cargo_target_dir,
        '--profile',
        cargo_profile,
    ]

    cargo_env = {
        'CARGO_HOME': cargo_home,
    }

    cargo_output = cargo_target_dir / rust_target / 'libglycin.so'

    # Build library with Rust
    libglycin = custom_target(
        'libglycin-cargo',
        build_always_stale: true,
        build_by_default: true,
        output: 'Meson-does-not-support-sub-dir-output',
        console: true,
        env: cargo_env,
        command: [cargo_bin, 'build', cargo_options, '-p', 'libglycin'],
    )

    # Make the build result available to meson. We cannot use the "output"
    # key in the previous target for this since meson prohibits output in subdirs
    libglycin_copy_library_for_deps = custom_target(
        'libglycin-copy-library2',
        depends: libglycin,
        build_by_default: true,
        build_always_stale: true,
        output: 'tmp.so',
        command: [
            'cp',
            '-a', cargo_output,
            meson.current_build_dir() / 'tmp.so',
        ],
    )

    # We need a shim "lib" object to pass to "generate_gir" later
    libglycin_shim = library(
        'glycin-1',
        'include/glycin.h',
        soversion: '0',
        dependencies: libglycin_deps,
        # Mark dirty for "generate_gir" if the header or the rust generated .so changes
        link_depends: [
            # For unkown reasons the header triggers repeated gir regenerates until something else changes
            'include/glycin.h',
            libglycin_copy_library_for_deps,
        ],
        install: true,
    )

    # Copy the .so file to its correct loaction such that "generate_gir" can use it.
    # We have to do this after the shim library target since it generates
    # a useless/empty library.
    libglycin_copy_library = custom_target(
        'libglycin-copy-library',
        depends: libglycin_shim,
        build_always_stale: true,
        build_by_default: true,
        output: 'would-overlap-with-library()',
        command: [
            'cp',
            '-a', cargo_output,
            libglycin_shim.full_path(),
        ],
    )

    install_headers(['include/glycin.h'], subdir: 'glycin-1')

    # Generate a dependency such that the following targets only happen
    # after the .so file has been copied to the correct place
    libglycin_copy_library_as_dep = declare_dependency(
        sources: libglycin_copy_library,
    )

    # Generate .pc file
    pkg.generate(
        name: 'glycin-1',
        description: 'Glycin: Sandboxed and extendable image decoding',
        libraries: libglycin_deps + libglycin_shim + libglycin_copy_library_as_dep,
        subdirs: 'glycin-1',
    )

    if introsepction
        # Generate .gir and .typelib
        libglycin_gir = gnome.generate_gir(
            libglycin_shim,
            dependencies: libglycin_copy_library_as_dep,
            sources: ['include/glycin.h'],
            nsversion: '1',
            namespace: 'Gly',
            symbol_prefix: 'gly',
            link_with: libglycin_shim,
            includes: ['GLib-2.0', 'GObject-2.0', 'Gio-2.0', 'Gdk-4.0'],
            # Listed as "C headers" in docs
            header: 'glycin.h',
            # Listed as "pkg-config files" in docs
            export_packages: 'libglycin-1',
            install: true,
        )
    endif

    if get_option('vapi')
        # Generate .vapi (Vala)
        gnome.generate_vapi(
            'libglycin-1',
            sources: libglycin_gir[0],
            packages: ['glib-2.0', 'gobject-2.0', 'gio-2.0', 'gtk4'],
            install: true,
        )
    endif

    if get_option('capi_docs')
        gidocgen = find_program('gi-docgen')

        # Generate HTML documentation
        custom_target(
            'libglycin-doc',
            # Generate from .gir file
            depends: libglycin_gir,
            input: ['doc/libglycin.toml'] + libglycin_gir[0],
            output: 'libglycin-1',
            command: [
                gidocgen,
                'generate',
                '--config=@INPUT0@',
                '--output-dir=@OUTPUT@',
                '--quiet',
                '--no-namespace-dir', '--content-dir=@0@'.format(
                    meson.current_source_dir() / 'doc',
                ),
                '@INPUT1@',
            ],
            build_by_default: true,
            install: true,
            install_dir: datadir / 'doc',
        )
    endif

    if get_option('python_tests')
        test('libglycin-python3', find_program('../tests/libglycin.py'))
    endif
endif
