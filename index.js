const worker = require('./entry');

exports.resolve = (source, file, options) => {
  const project = Array.isArray(options.project) ? options.project[0] : options.project;
  return worker.resolve(source, file, project);
};

exports.interfaceVersion = 2;
