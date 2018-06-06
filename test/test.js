'use strict';

var expect = require('chai').expect;
var Idea = require('../dist/Idea.js').Idea;

describe('Single Idea progress test', () => {
  var idea = new Idea('Idea without children');
  it('should start at 0', () => {
    expect(idea.progress).to.equal(0);
  });
  it('should change when set', () => {
    idea.progress = 0.5;
    expect(idea.progress).to.equal(0.5);
    idea.progress = 1;
    expect(idea.progress).to.equal(1);
  });
  it('should error when set negative', () => {
    expect(() => { idea.progress = -1; }).to.throw();
  });
  it('should error when set >100%', () => {
    expect(() => { idea.progress = 2; }).to.throw();
  });
});

describe('Divided Idea progress test', () => {
  var idea = new Idea('Idea with children');
  var child1 = idea.addChild('Child 1');
  var child2 = idea.addChild('Child 2');

  it('should start at 0', () => {
    expect(idea.progress).to.equal(0);
  });
  it('should error when set directly', () => {
    expect(() => { idea.progress = 1; }).to.throw();
  });

  it('should reflect the progress of its children', () => {
    child1.progress = 0.5;
    expect(idea.progress).to.equal(0.25);
    child2.progress = 0.5;
    expect(idea.progress).to.equal(0.5);
    child1.progress = 1;
    expect(idea.progress).to.equal(0.75);
    child2.progress = 1;
    expect(idea.progress).to.equal(1);
  });
});
