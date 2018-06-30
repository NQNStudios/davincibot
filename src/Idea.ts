import * as db from 'quick.db';

type IdeaData = {
    id: number;
    name: string = '';
    description: string = '';
    tags: Array<string> = [];
    childIds: Array<number> = [];
}

// The basic unit of user data in Da Vinci Bot.
// TODO write clear documentation on the Idea metaphor/design pattern
export class Idea {
    readonly data: IdeaData;

    // TODO add properties for direct access to data fields?

    // TODO allow pre-initialization of child Ideas as objects with a recursion
    // depth specifier

    private constructor(id: number) {
        this.id = id;
    }

    static async Initialize(): void {
        let ideaCount = await db.fetch('IdeaCount');
        if (ideaCount === null) {
            db.set('IdeaCount', 0);
        }
    }

    static async Get(id: number) {
        let idea = await db.fetch(id.toString());
    }
};

export class Ideas
{
    static async NewIdea(): Idea {
        let ideaCount = await db.fetch('IdeaCount');
        db.add('IdeaCount', 1);

        let newIdea: Idea = { id: ideaCount }; 
        return newIdea;
    }
}

Ideas.Initialize();
