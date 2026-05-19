// TypeORM が CommonJS の .cjs を含むため Metro の sourceExts に追加する
const { getDefaultConfig } = require('expo/metro-config');
const config = getDefaultConfig(__dirname);
config.resolver.sourceExts.push('cjs');
module.exports = config;
