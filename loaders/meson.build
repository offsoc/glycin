cargo_bin = find_program('cargo')

if get_option('profile') == 'release'
  rust_target = 'release'
  profile = 'release'
else
  rust_target = 'debug'
  profile = 'dev'
endif

test_skip_ext = ''

foreach ext : get_option('test_skip_ext')
  if test_skip_ext != ''
    test_skip_ext += ','
  endif
  test_skip_ext += ext
endforeach

target_dir = meson.project_build_root() / 'cargo_target'
manifest = meson.project_source_root() / 'Cargo.toml'

cargo_options = [
  '--profile', profile,
  '--manifest-path', manifest,
]

cargo_env = {
  # Don't use $HOME/.cargo/ for this
  'CARGO_HOME': meson.project_build_root() / 'cargo',
  # Gettext translation
  'LOCALEDIR': get_option('prefix') / get_option('localedir'),
  # Git commit for debugging
  'GIT_DESCRIBE': run_command(
      'git',
      '-C', meson.project_source_root(),
      'rev-parse', '--short', 'HEAD',
      check: false
    ).stdout().strip(),
}

test_args = []
foreach loader : get_option('loaders')
  test_args += ['-p', loader]

  cargo_build = custom_target(
    loader,
    build_by_default: true,
    build_always_stale: true,
    output: loader + '-bin',
    console: true,
    env: cargo_env,
    command: [
      cargo_bin, 'build',
      '--target-dir', target_dir / 'loaders',
      [cargo_options, [ '--package', loader ]],
    ],
  )

  custom_target(
    loader + '-cp-binary',
    depends: cargo_build,
    build_by_default: true,
    build_always_stale: true,
    install: true,
    install_dir: libexecdir,
    output: loader,
    command: [
      'cp',
      target_dir / 'loaders' / rust_target / loader,
      '@OUTPUT@',
    ],
  )

  i18n.merge_file(
      type: 'desktop',
      input: configure_file(
          input: loader / loader + '.conf',
          output: 'tmp-' + loader,
          configuration: { 'EXEC': libexecdir / loader }
      ),
      output: loader + '.conf',
      po_dir: '..' / 'po',
      install: true,
      install_dir: glycin_datadir / 'conf.d'
  )
endforeach

if get_option('tests')
  destdir = meson.project_build_root() / '_test-install'

  if not get_option('test_skip_install')
    test(
      'install-integration-test',
      find_program('meson'),
      args: [
        'install',
        '-C',
        meson.project_build_root(),
        '--destdir',
        destdir,
      ],
      is_parallel: false,
    )

    # Rewrite config to point into destdir
    setup_integration_test = find_program(meson.project_source_root() / 'build-aux/setup-integration-test.py')
    foreach loader : get_option('loaders')
      test(
        'setup-integration-test-' + loader,
        setup_integration_test,
        env: {
          'DESTDIR': destdir,
          'CONFIG_FILE': glycin_datadir / 'conf.d' / loader + '.conf',
        },
        is_parallel: false,
      )
    endforeach
  endif

  env = { 'GLYCIN_TEST_SKIP_EXT': test_skip_ext }

  if not get_option('test_skip_install')
    # Use string concatenation to combine two absolute paths
    env += { 'GLYCIN_DATA_DIR': destdir + datadir  }
  endif

  test(
    'cargo-test',
    cargo_bin,
    args: [
      'test',
      '-p', 'tests',
      test_args,
      '--target-dir', target_dir / 'tests',
      cargo_options,
      '--',
      '--test-threads=1',
      '--nocapture',
    ],
    env: env,
    is_parallel: false,
    timeout: 1800,
  )
endif