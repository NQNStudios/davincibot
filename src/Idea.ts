// The basic unit of things the user wants to do. Ideas are all created in
// unstructured form, and structured later by dividing and subdividing them
// into smaller Ideas.
export class Idea
{
    // Each Idea will have a unique id numbered upwards from 0.
    // TODO this will need to be serialized/deserialized when Tasks are reloaded from
    // a previous session!
    static Count: number = 0;

    id: number;
    name: string;
    description: string = '';
    tags: { [id: string]: boolean; } = {};

    children: Array<Idea> = [];
    private _progress: number = 0;

    // Create an unstructured Idea
    constructor(name: string) {
        this.id = Idea.Count++;
        this.name = name;
    }

    // Check how near this Idea is to tangible completion
    get progress(): number {
        // Only Ideas without children can define their own progress
        if (this.children.length == 0) {
            return this._progress;
        }
        // If an Idea has children, its progress is measured as the average of
        // its children's progress values
        else {
            let sum = 0;
            for (let i = 0; i < this.children.length; ++i) {
                let child = this.children[i];
                sum += child.progress;
            }
            return sum / this.children.length;
        }
    }

    // Set how near this Idea is to tangible completion --
    // Only valid for an undivided Idea
    set progress(value: number) {
        if (this.children.length == 0) {
            if (value >= 0 && value <= 1) {
                this._progress = value;
            }
            else {
                throw new Error(`Tried to set an Idea's progress out of bounds [0:1]: ${value}`);
            }
        }
        else {
            throw new Error('Tried to directly set the progress of a task with children.');
        }
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
