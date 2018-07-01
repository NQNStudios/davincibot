import * as db from 'quick.db';

// The basic unit of user data in Da Vinci Bot.
// TODO write clear documentation on the Idea metaphor/design pattern
export type IdeaData = {
    readonly id: number;
    name: string;
    description: string;
    tags: Array<string>;
    childIds: Array<number>;
};

async GetIdea(id: number): IdeaData {
    let idea = await db.fetch(id.toString());
    return idea;
}

async CreateIdea(): IdeaData {
    let ideaCount = await db.fetch('IdeaCount');
    db.add('IdeaCount', 1);

    return { id: ideaCount, name: '', description: '', tags: [], childIds: [] };
}

async SetName(id: number, name: string) {
    return await db.set(id.toString(), name, {target: ".name"});
}

async SetDescription(id: number, description: string) {
}

async AddTag(id: number, tag: string) {
}

async AddChild(id: number, child: IdeaData) {
}

    static async Get(id: number) {
        let idea = await db.fetch(id.toString());
    }
}

let ideaCount = await db.fetch('IdeaCount');
if (ideaCount === null) {
    db.set('IdeaCount', 0);
}
