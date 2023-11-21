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

foreach loader : get_option('loaders')
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

if not get_option('test_skip_install')
  test(
    'test-install',
    find_program('meson'),
    args: ['install', '-C', meson.project_build_root()],
  )
endif

test(
  'cargo-test',
  cargo_bin,
  args: [
    'test',
    '--workspace',
    '--target-dir', target_dir / 'tests',
    cargo_options,
    '--',
    '--test-threads=1',
    '--nocapture',
  ],
  env: { 'XDG_DATA_HOME': datadir, 'GLYCIN_TEST_SKIP_EXT': test_skip_ext },
  is_parallel: false,
  timeout: 1800,
)