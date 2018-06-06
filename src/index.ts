import { Idea } from "./Idea";
import { JsonConvert } from "json2typescript";

let recursiveIdea = new Idea("Top-level idea");

let middleIdea = recursiveIdea.addChild("middle-level child");

let finalIdea = middleIdea.addChild("bottom-level child");

let converter = new JsonConvert();

let jsonString = JSON.stringify(recursiveIdea);

let deserializedIdea = Idea.ParseString(jsonString);

console.log(jsonString);
console.log(deserializedIdea);
