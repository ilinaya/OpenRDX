import {NgModule} from '@angular/core';
import {provideRouter} from '@angular/router';
import {routes} from './login.routes';

@NgModule({
  providers: [provideRouter(routes)],
})
export class LoginModule {
}
