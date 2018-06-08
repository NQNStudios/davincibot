import { Duration, duration } from "moment";
import { JsonObject, JsonProperty } from "json2typescript";

// The basic unit of things the user wants to do. Ideas are all created in
// unstructured form, and structured later by dividing and subdividing them
// into smaller Ideas.
@JsonObject
export class Idea
{
    // Index definition to allow key-based access to attributes
    [key: string]: any;

    // Each Idea will have a unique id numbered upwards from 0.  This doesn't
    // need to be serialized because each deserialization will increment count
    // in the constructor before restoring the Idea's original id
    static TotalCount: number = 0;
    static None: Idea = new Idea();

    @JsonProperty("id", Number)
    id: number;

    @JsonProperty("name", String)
    name: string;

    @JsonProperty("description", String)
    description: string = '';

    @JsonProperty("tags")
    tags: { [id: string]: boolean; } = {};

    @JsonProperty("children")
    children: Array<Idea> = [];

    @JsonProperty("_progress", Number)
    private _progress: number = 0;
    @JsonProperty("_duration")
    private _duration: Duration = duration(0);

    // Create an unstructured Idea
    constructor(name: string = '') {
        this.id = Idea.TotalCount++;

        this.name = name;
    }

    // Calculate an attribute that is determined by either the sum or average
    // of that attribute in this Idea's children
    private getOverallAttribute(attribute: string, sumZero: any = 0, useAverage: boolean): any {
        if (this.children.length == 0) {
            return this[`_${attribute}`];
        }
        else {
            let sum = sumZero;
            for (let i = 0; i < this.children.length; ++i) {
                let child = this.children[i];
                sum += child[attribute];
            }

            let value = sum;
            if (useAverage) {
                value /= this.children.length;
            }

            return value;
        }
    }

    private setOverallAttribute(attribute: string, value: any) {
        if (this.children.length == 0) {
            this[`_${attribute}`] = value;
        }
        else {
            throw new Error(`Tried to directly set the ${attribute} of a task with children.`);
        }
    }

    // Check how near this Idea is to tangible completion
    get progress(): number {
        return this.getOverallAttribute('progress', 0, true) as number;
    }

    // Set how near this Idea is to tangible completion --
    // Only valid for an undivided Idea
    set progress(value: number) {
        if (value < 0 || value > 1) {
            throw new Error(`Tried to set an Idea's progress out of bounds [0:1]: ${value}`);
        }
        else {
            this.setOverallAttribute('progress', value);
        }
    }

    get duration(): Duration {
        return this.getOverallAttribute('duration', duration(0), false) as Duration;
    }

    set duration(value: Duration) {
        this.setOverallAttribute('duration', value);
    }

    get remainingDuration(): Duration {
        let totalSeconds = this.duration.asSeconds();
        let completeSeconds = this.progress * totalSeconds;
        return duration(totalSeconds - completeSeconds, 'seconds');
    }

    // Create a new Idea as a child of this one
    addChild(name: string) {
        let newChild = new Idea(name); 
        this.children.push(newChild);
        return newChild;
    }

    addTag(tag: string) {
        this.tags[tag] = true;
    }

    removeTag(tag: string) {
        this.tags[tag] = false;
    }

    toggleTag(tag: string) {
        this.tags[tag] = !this.hasTag(tag);
    }

    hasTag(tag: string): boolean {
        return tag in this.tags && this.tags[tag];
    }

    // Check if this Idea has a set of given tags (all, some, or none are all
    // checked at once)
    hasTags(tagsToSearch: Array<string>): HasTagsResult {
        let result = new HasTagsResult();

        for (let i = 0; i < tagsToSearch.length; ++i) {
            let tag = tagsToSearch[i];

            if (this.hasTag(tag)) {
                result.some = true;
            }
            else {
                result.all = false;
            }
        }

        result.none = !result.some;
        return result;
    }

    become(otherIdea: Idea) {
        this.id = otherIdea.id;
        this.children = otherIdea.children;
    }
}

// A set of results for the Idea.hasTags() function, each according to
// a different criterion
export class HasTagsResult
{
    // Does this Idea have ALL of the given flags?
    all: boolean = true;
    // Does this Idea have SOME of the given flags?
    some: boolean = false;
    // Does this Idea have NONE of the given flags?
    none: boolean = false;
}
