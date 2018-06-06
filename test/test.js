'use strict';

var expect = require('chai').expect;
var index = require('../dist/index.js');

describe('failing test', () => {
  it('should fail', () => {
    expect(5).to.equal(9);
  });
});
