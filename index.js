const worker = require('./entry');

exports.resolve = (source, file, options) => {
  const project = Array.isArray(options.project) ? options.project : [options.project];
  return worker.resolve(source, file, { project });
};

exports.interfaceVersion = 2;
