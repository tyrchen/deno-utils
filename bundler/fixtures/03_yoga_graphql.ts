import { createServer } from './yoga.ts';

const yoga = createServer({
  schema: {
    typeDefs: /* GraphQL */ `
      scalar File
      type Query {
        hello: String
      }
      type Mutation {
        getFileName(file: File!): String
      }
      type Subscription {
        countdown(from: Int!): Int!
      }
    `,
    resolvers: {
      Query: {
        hello: () => 'world',
      },
      Mutation: {
        getFileName: (_root: any, { file }: { file: File }) => file.name,
      },
      Subscription: {
        countdown: {
          async *subscribe(_: any, { from }: any) {
            for (let i = from; i >= 0; i--) {
              await new Promise((resolve) => setTimeout(resolve, 1000));
              yield { countdown: i };
            }
          },
        },
      },
    },
  },
  logging: false,
});

console.log(yoga);
