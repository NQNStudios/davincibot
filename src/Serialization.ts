import {Idea} from "./Idea";
import {BotProcess, BotStatus} from "./DaVinciBot";
import {JsonConvert} from "json2typescript";

export class LoadFileProcess extends BotProcess
{
    description(): string { return  'Load ideas from a file.'; }
}

export class SaveFileProcess extends BotProcess
{
    description(): string { return  'Save ideas to a file.'; }
}

export class LoadProcess extends BotProcess
{
    static converter = new JsonConvert();

    description(): string { return 'Load ideas from a JSON string.'; }

    start(rootIdea: Idea): BotStatus {
        return BotStatus.NeedsInput;
    }

    handleInput(input: string, rootIdea: Idea) {
        let jsonObject: object = JSON.parse(input);
        let newRootIdea = LoadProcess.converter.deserializeObject(jsonObject, Idea);
        rootIdea.become(newRootIdea);

        // TODO need to somehow set Idea.TotalCount to the total count--
        // recursion isn't working

        return BotStatus.Idle;
    }

    finish(rootIdea: Idea): void {
        // TODO: Don't always do this
        console.log(rootIdea.children);
    }
}

export class SaveProcess extends BotProcess
{
    description(): string { return 'Save ideas to a JSON string.'; }

    start(rootIdea: Idea): BotStatus {
        return BotStatus.HasOutput;
    }

    getOutput(rootIdea: Idea): [string, BotStatus] {
        return [JSON.stringify(rootIdea), BotStatus.Idle];
    }
}
