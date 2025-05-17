import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ReactiveFormsModule } from '@angular/forms';
import { SettingsRoutingModule } from './settings-routing.module';
import { SettingsComponent } from './settings.component';
import { AdminListComponent } from './components/admin-list/admin-list.component';
import { AdminDetailComponent } from './components/admin-detail/admin-detail.component';
import { AdminFormComponent } from './components/admin-form/admin-form.component';
import { AdminGroupListComponent } from './components/admin-group-list/admin-group-list.component';
import { AdminGroupDetailComponent } from './components/admin-group-detail/admin-group-detail.component';
import { AdminGroupFormComponent } from './components/admin-group-form/admin-group-form.component';
import { SecretListComponent } from './components/secret-list/secret-list.component';
import { SecretDetailComponent } from './components/secret-detail/secret-detail.component';
import { SecretFormComponent } from './components/secret-form/secret-form.component';
import { VendorListComponent } from './components/vendor-list/vendor-list.component';
import { VendorFormComponent } from './components/vendor-form/vendor-form.component';
import { VendorDetailComponent } from './components/vendor-detail/vendor-detail.component';
import { ChangePasswordComponent } from './components/change-password/change-password.component';

@NgModule({
    declarations: [
        SettingsComponent,
        AdminListComponent,
        AdminDetailComponent,
        AdminFormComponent,
        AdminGroupListComponent,
        AdminGroupDetailComponent,
        AdminGroupFormComponent,
        SecretListComponent,
        SecretDetailComponent,
        SecretFormComponent,
        VendorListComponent,
        VendorFormComponent,
        VendorDetailComponent,
        ChangePasswordComponent
    ],
    imports: [
        CommonModule,
        ReactiveFormsModule,
        SettingsRoutingModule
    ]
})
export class SettingsModule { }
