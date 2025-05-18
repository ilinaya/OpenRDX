import {NgModule} from '@angular/core';
import {routes} from './dashboard.routes';
import {provideRouter} from '@angular/router';

@NgModule({
  providers: [provideRouter(routes)],
})
export class DashboardModule {
}
